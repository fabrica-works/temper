# temper's client

Provisional, 2026-10-14, revised 2026-10-19. temper's client: temper's
own presentation on jig's client, `jig-client`. Its crates; its client
root, which holds `jig-client` and temper's pages; the forge's client
family, its words and its one form; its view, on skein's UI kit; the
terminal that `temper open` runs, on skein's text shell; signing in and
the small store; what temper's server does for it; and how it is
tested. jig's client is jig's `client/domain/client.md`, its API jig's
`client/domain/contract.md` and its protocol jig's
`protocol/client.md`: this document takes them as given and cites them.
The forge's parts of the client are `forge.md`, the browser
`browser.md`. The mockups (`mockups/`) are reference material for the
pages' look and the order of their parts, not contract. What is still
open is listed in section 11.

## 1. In one page

- **temper's presentation, on jig's client.** temper's client root
  holds `jig-client` as a component (jig's `client/domain/root.md`):
  the link, watches by handle, keyed requests until durable, the work
  kept across a restart, and the forge's family carried as bytes. Its
  pages, words, confirmations and navigation are temper's own. jig's
  reference UI (jig's `client/domain/pages.md` and jig's
  `client/domain/view.md`) is not temper's: temper arranges no page of
  jig's, and copies no client root, view root, native client or card of
  jig's.
- **Its pages,** on jig's (jig's `client/domain/pages.md`): the inbox
  (home), the chats, a chat, a project's board, a task's page, the
  changes, and the forge's adoption form (3.5). The mockups give their
  look and the order of their parts. What no page shows is open
  (section 11).
- **Its cards are its own.** A proposal, a question, a held task, a
  person task, a result, an entry of activity, an outcome and each
  confirmation are temper's nodes, drawn from the object's document as
  `jig-client` delivers it and from `jig-client`'s `facts` (jig's
  `client/domain/client.md`, section 19), as jig's
  `client/domain/objects.md`, section 2 allows an application. temper
  calls no renderer of jig's reference UI, and copies none of its
  modules, functions or phrase tables.
- **On jig's API alone.** What a page shows, it watches through jig's
  subjects (`me`, `inbox`, `tasks{query}`, `task`, `tree`, `run`) or
  reads (`activity`, `proposal`; jig's `client/domain/contract.md`,
  sections 6 and 7); what a person does is jig's operations (jig's
  `client/domain/contract.md`, section 3), through jig's helpers where
  they fit (3.6).
- **The confirmation is temper's.** An act that cannot be undone says
  what it will do before it is sent, from the facts jig's documents
  give, and every deciding act names the revision the person was shown
  (3.6).
- **The forge's family.** Its words for the connector, a repository, a
  branch, a pull request and each kind of write need no document (4.1).
  Its other parts are documents its client side reads, with requests of
  their own (`forge.md`); a part it does not read says "Shown on the
  forge." (4.2). Its form is the adoption, sent through `jig-client`'s
  request table as the forge's bytes, like any other keyed request
  (4.3 to 4.7).
- **One crate of records.** `temper-client-protocol-wire` carries the
  forge family's records: the adoption, its outcome and the small
  store's record of a pending one (4.6), and those of `forge.md`,
  section 9. temper's server decodes them before the engine's root,
  which locates the repository and adopts it on the person's behalf
  (section 8).
- **One view, every shell.** `temper-client-view` builds temper's pages
  as skein-ui's tree (skein's `ui.md`); the terminal prints that tree
  with skein's text shell (skein's `ui-text.md`); the browser, a later
  shell, maps it with skein's browser parts (`browser.md`).
- **A terminal first.** `temper open` runs temper's client, its view and
  skein's text shell over `jig-client-shell-native`, jig's native
  transport (section 6).
- **Signed in by jig's printed link,** on loopback, and kept signed in
  across restarts by its bearer credential, which the terminal keeps in
  the client's private directory (section 7). The forge as people's
  sign-in provider is open (section 11).
- **temper's server owns the clients' listener,** and mounts jig's
  clients' component on it with jig's default HTTP mapping, under
  `/jig`, with its example guards (section 8).
- **Joined to the engine from the start.** `tests/client`
  (`temper-client-world`) runs temper's client and view, with skein's
  fake person, against temper's engine root (section 10;
  `../testing.md`, 6.3).

## 2. The client's crates

```
temper's client
temper-client-domain            the client root: jig-client held, temper's pages, the forge's side (sections 3 and 4)
├── jig-client                  jig's client: the link, watches by handle, the request table, kept work, families; jig's helpers
└── temper-client-protocol-wire the forge family's records, for both ends (4.6)
temper-client-view              the view: temper's pages as skein-ui's tree, temper's words (section 5)
└── skein-ui                    skein's UI kit: the tree, the builder, bindings, the diff, phrasebooks

below the client
jig-client-protocol-wire        jig's records, for both ends, under jig-client
jig-protocol-transcript         turns decoded with smith's transcript crate, under jig-client

the shell
temper-client-shell-terminal    the terminal: temper's host, its settings and lines (section 6)
├── skein-ui-text               skein's text shell: the renderer, the command reader, the terminal
└── jig-client-shell-native     jig's native transport: skein's HTTP client with jig-client-protocol-http; the small store's file and the credential's
temper                          the binary; `temper open` runs the terminal (../protocol/README.md, section 4)

for agents
temper-client-tools             the agents' tools on jig-client, beside the view (../tools.md, section 4)
temper-client-shell-cli         `temper ctl`, over jig-client-shell-native (../tools.md, section 8)

for tests
skein-fake-person               the person, on skein-ui's tree (../testing.md, 4.4)
jig-client-test-kit             jig's check, and the faults of jig's client worlds (jig's `client/domain/testing.md`, 6.2 and section 7)
jig-client-fake-person-browser  jig's attack stories (jig's `protocol/browser.md`, section 15)
temper-client-world             tests/client: temper's client joined to temper's engine root (section 10)
```

- **The crate graph.** `temper-client-domain` depends on skein-lib,
  `jig-client`, jig's records (`jig-client-protocol-wire`,
  `jig-client-protocol-link`) and `temper-client-protocol-wire`;
  `temper-client-view` on skein-lib, `skein-ui`, `jig-client` (for jig's
  facts and words) and `temper-client-domain`, and on no crate of jig's
  reference UI. Neither links io, HTTP, smith, or an engine crate of
  temper's: `jig-client` emits jig's messages, never HTTP, and turns
  reach it decoded (jig's `client/domain/client.md`).
- **`temper-client-protocol-wire`** is a codec crate generated by
  skein's generator from a schema, depending on skein-lib and
  skein-codec alone, no crate of jig's (jig's `protocol/client.md`,
  sections 3 and 11). temper's server and the client both use it.
- **`temper-client-shell-terminal`** holds, as modules:
  - `client`: the native client, `NativeClient`: temper's client root,
    its view and `jig-client-shell-native`'s transport, and when
    `jig-client-shell-native` keeps the credential's file (section 7);
  - `route`: each of the root's requests to the native shell, the host
    or one of temper's lines;
  - `terminal`: skein's text shell's terminal with temper's settings,
    its own lines and the sign-in's retries (6.2, 6.6, 6.7); and
    `Terminal`, the one host of skein's loop, composing the native
    client with stdin, stdout and the termination signals (6.2). The
    worlds put skein's fake person where `temper open` puts the
    terminal (section 10);
  - `limits`: the client's limits and their sum.
- **Step code under the subset:** the domain, the forge's side, the
  view and the wire. **Ordinary Rust:** the terminal's host, as a shell
  is (skein's `ui.md`, section 2).
- **The client's limits** are constants of
  `temper-client-shell-terminal`: `jig-client`'s, from jig's small
  profile with the overrides temper's configuration gives the engine's
  documents (`../protocol/configuration.md`, section 7); the pages' and
  the forge side's; skein-ui's, derived from the view's (5.4); and their
  sum. The client reads no configuration: its only inputs are its two
  arguments (`../protocol/configuration.md`, section 9).

## 3. The client root

`temper-client-domain` is temper's client root, in jig's one shape
(jig's `client/domain/root.md`, section 3): `new`, `step`, `fire`,
`next_deadline` and `take`, and `shown`, a count of what is shown,
which the view builds from. It holds `jig_client::Client` as a
component and calls it; the client never calls it back, and the root
routes nothing of jig's (jig's `client/domain/client.md`, section 4).

### 3.1 What it holds

- **`jig-client`,** given its limits and a seed at construction.
- **Two children, each an owner** (jig's `client/domain/root.md`,
  section 15): temper's pages, owner `Pages`, which ask the client for
  jig's operations, reads and subjects only; and the forge's client
  side, owner `Forge` (section 4); numbered in 4.6. Each is the asker
  of every handle and key it gives `jig-client`, and draws its keys'
  numbers from the seed the shell gives the root.
- **`me`,** watched from start to sign-out under a handle of its own:
  the signed-in party, their projects with their roles and pools, what
  waits for them counted, and each project's merged templates (jig's
  `client/domain/contract.md`, 5.1). Every page shows the link's state,
  `jig-client`'s `Link`.
- **The page at its address** (3.5): its handles, its windows, the
  composer's words, and the open confirmation (3.6), at most one.
- **The project the person is in,** chosen among `me`'s projects; the
  first at start.
- **The pages' acts in flight,** each by its key: which page's card or
  composer sent it, so its answer reaches what shows it (3.3).

Everything is bounded by the domain's limits: the handles a page opens,
the rows of each window, the composer's bytes, the acts in flight.

### 3.2 Its modules

```
temper-client-domain/src/
├── lib.rs          the tree, re-exports
├── boundary.rs     events in (the start, a place gone to, the person's acts, jig-client's events), requests out (jig-client's, a place shown, signing out, kept work dropped, facts)
├── owner.rs        temper's two owners, Pages and Forge, as askers (3.3)
├── domain.rs       Domain: new, step, fire, next_deadline, take, shown; jig-client held
├── route.rs        each event to its child; what jig-client tells, to the child its asker names (3.3)
├── address.rs      temper's addresses and the inbox's filters (3.5)
├── pages.rs        the pages: their handles, the project the person is in, navigation (3.1)
├── pages/          one module per page: what it watches and reads, its state, its acts
├── acts.rs         acts as jig's operations, through jig's helpers; the confirmation (3.6)
├── forge.rs        the forge's client side (section 4), with its modules below forge/
├── limits.rs       jig-client's limits, the pages', the forge side's, and their sum
└── tests.rs
```

### 3.3 Routing

As jig's client root routes (jig's `client/domain/root.md`, section 5),
with temper's two children:

- **Out and in:** `jig-client`'s requests to the transport leave the
  domain as they are, for the shell to carry; the transport's events
  reach `jig-client` as they are.
- **By asker:** what `jig-client` says of a handle or of a key goes to
  `Pages` or `Forge`, matched whole; an asker that names neither is a
  fact (`UnknownAsker`), and reaches no child. The pages hand it to the
  page that opened the handle, or the card or composer that sent the
  key, and drop one for a handle they closed; the forge's side to its
  form.
- **The link** goes to both children, and changes what every page
  shows; a page reopens nothing of its own, since `jig-client` keeps the
  watches. Kept work dropped goes to the shell (3.4).
- **A place gone to** goes to the pages, and to the forge's side when
  it is `Adopt { project }`; any other tells the forge's side it left.

### 3.4 At start

The shell starts the domain with the small store's records and the
inbox's address, then signs in (section 7), holding back neither. The
domain's children ask their watches from their start, `me` among them,
and `jig-client` opens none on the engine until it is signed in (jig's
`client/domain/client.md`, section 12), so no stream leaves without the
credential.

- **Another party's kept work** stays parked and is not shown (jig's
  `client/domain/client.md`, section 11); the terminal says so in a line
  of its own (6.6).
- **Kept work dropped:** the domain hands `jig-client`'s `Dropped {
  count }` to the shell (jig's `client/domain/root.md`, section 8), and
  the terminal says so in a line of its own (6.6).
- **temper's pages keep no record of their own:** a client closed on
  any page opens at the inbox, and a composer's words not sent, or a
  form's values, are not kept across a restart (section 11).

### 3.5 Its pages

Each page has an address, and watches what it shows while it is open;
leaving it closes its handles. Its mockup gives its look and the order
of its parts (reference material, not contract). Lists of tasks are
task queries (jig's `client/domain/contract.md`, 6.1): the chat and
goal templates' labels are jig's defaults, which temper's configuration
keeps (`../protocol/configuration.md`, section 2).

| Address | Mockup | Shows | Watches and reads | Acts |
|---|---|---|---|---|
| `Inbox` (home) | `inbox.html` | what needs the person, across projects, filtered by project, kind and goal (its root): proposals, questions, held tasks, person tasks; then updates (results, replies) | `inbox` with its filters; the `proposal` or `task` an entry's card needs | accept, reject with a reason, answer, release, leave held, take, hand back; seen |
| `Chats` | `chats.html` | a composer to start a chat; the person's chats, live or closed, newest activity first, each with its status, spend and delegates | `tasks{query}`: requester the person, the chat label, by last activity | start a chat |
| `Chat { task }` | `index.html` | the conversation: committed turns, the person's words, work summaries, provisional text; the proposals waiting in its subtree, each at the call that made it where its turn names it, and after the conversation otherwise (jig's `client/domain/objects.md`, section 10); its status and spend; Stop and Close; the composer | `task`, `activity` a page at a time, `run` while one is live; each `proposal` its `task` names | message, stop, close, accept or reject a proposal in place; seen |
| `Board { project }` | `board.html` | the project's goals in priority order, live, waiting and recently ended, with their progress, spend and deadlines; what needs the person; "Set a goal" | `tasks{query}`: the project, tracked, by priority; ended, by ended at | set a goal, reorder goals |
| `Task { task }` | `task.html`, `agent-task.html` | a goal: its header, what was asked, its plan as a tree, its latest activity and history, its budget and authority, with Amend, Stop and Cancel; an agent's task: its transcript and a composer, with Amend and Stop | `task`, `tree`, `activity`; `run` while one is live | amend, stop, cancel, release a held task of the plan, message an agent's task |
| `Changes { project }` | `changes.html` | the forge's changes and landing queues (`forge.md`, section 3) | the forge's, through the forge's side (4.2) | the forge's |
| `Adopt { project }` | none: 4.4 | the adoption form | nothing but `me` | adopt a repository (4.4) |

- **Words and facts:** a chat's status from jig's facts (jig's
  `client/domain/client.md`, section 19), in jig's words, with temper's
  own where the mockups word it otherwise ("Waiting for you",
  "Parked"); a conversation from jig's thread model; jig's enums and a
  decision's facts in jig's words, under temper's phrasebook (5.1);
  each card temper's own (section 1).
- **A task's page** is of its kind: a tracked task's, as `task.html`
  lays it out; an agent's task's, as `agent-task.html`. A task with the
  chat label opens as `Chat { task }`. A procedure's task or a person
  task shows a tracked task's header and plan only (section 11).
- **The forge's parts** of a page (a goal's issue and changes on its
  page; changes in flight and the project's health on the board) are
  `forge.md`'s, and "Shown on the forge." while the forge's side does
  not read them (4.2).
- **Links are by number:** a chip, an entry or a line naming a task
  opens its page, from any page (jig's `client/domain/pages.md`,
  section 2).

### 3.6 Requests and confirmations

- **One act, one request,** keyed by the domain and sent through
  `jig-client`, which keeps it until its answer is durable (jig's
  `client/domain/client.md`, section 6): a double press joins the
  pending one, and nothing is shown done before its answer.
- **jig's operations, through jig's helpers** (jig's
  `client/domain/client.md`, section 19; jig's
  `client/domain/contract.md`, 3.2 and 3.4): starting a chat is
  `shapes::chat`; setting a goal is `shapes::set_goal`, a proposal where
  the person's authority does not cover it; closing a chat is
  `shapes::close_chat`, one compound request, with what becomes of each
  live delegate; reordering goals is one compound request of priority
  changes; amending and releasing a held task is one compound request.
  Every other act is one operation.
- **Said before it is sent.** An act that cannot be undone (cancelling,
  closing a chat, accepting, rejecting, releasing) says what it will do
  before it is sent, from the facts the document gives (a decision's
  facts, as `jig-client`'s `facts` reads them): what accepting grants
  and costs and the pool after it, what cancelling ends, what releasing
  does (jig's `client/domain/objects.md`, section 1; jig's
  `client/domain/contract.md`, 5.4). Where the document offers an act
  as proposable (jig's `client/domain/contract.md`, 5.3), the card says
  so, and to whom; where it offers it as refused, the card says why and
  sends nothing.
- **It names what the person saw.** Each deciding act names the
  revision of the document the card showed; when the engine refuses it
  as moved, the card asks the person to look again (jig's
  `client/domain/contract.md`, 5.2). jig's check holds it in temper's
  worlds (section 10).
- **Answered:** the card shows the outcome, or the refusal in jig's
  words with what fits (look again, change what was asked); unknown,
  past the engine's retention, it says so.

## 4. The forge's client family

The forge's client side is the client root's child for the forge
connector, an owner numbered as its connector (jig's
`client/domain/root.md`, sections 2 and 15). It shows the forge's parts
that need no document, reads the documents of the others (`forge.md`),
and holds one form, the adoption.

### 4.1 Words that need no document

A connector, a resource or a kind needs no document: its words come from
its number or its path.

- **The connector:** "forge".
- **A resource,** by its path (`../domain/forge.md`, section 2):

  | Path | Words |
  |---|---|
  | `forge` HOST OWNER NAME | OWNER/NAME on HOST |
  | … `branch` S1 … Sn | branch S1/…/Sn of OWNER/NAME on HOST |
  | … `pull` N | pull request #N of OWNER/NAME on HOST |
  | … `issue` N | issue #N of OWNER/NAME on HOST |
  | two segments: a forge's number (2 bytes) and a repository's id (4 bytes), big-endian, as an adoption names it | repository ID on forge F |
  | anything else | its segments joined by `/`, as they are |

  jig's documents name an adopted repository by its forge's number and
  its id (`../domain/forge.md`, section 4), and the client shows those;
  the forge's documents name it by owner and name (`forge.md`, 2.5).
- **A resource pattern** (a grant's): the words of its segments as
  above, its last segment as it is; an open end is its prefix followed
  by `…` ("branch temper/7/… of acme/shop on forge.example").
- **A kind,** by the forge's numbering (`temper-engine-domain-forge`'s
  `effect_shape`; `../domain/forge.md`, section 6):

  | Kind | Words | Kind | Words |
  |---|---|---|---|
  | 1 | read | 9 | create a branch |
  | 2 | push | 10 | edit a pull request |
  | 3 | open a pull request | 11 | update a pull request from its base |
  | 4 | merge a pull request | 12 | close or reopen a pull request |
  | 5 | post a review | 13 | edit, close or reopen an issue |
  | 6 | set a commit status | 14 | set reviewers |
  | 7 | post a comment | 15 | delete a branch |
  | 8 | create an issue | any other N | kind N |

  These are the forge client's writes in words (its `Write`
  vocabulary), with reading and pushing. A new kind is a new row here
  in the same change that numbers it.

### 4.2 The forge's other parts

Every other part of the forge's that jig's documents name (a panel, a
result, an effect, an outbox entry, a hold, a project's part, the
deployment's part: jig's `client/domain/root.md`, section 6) needs a
document, which the forge's side reads or watches through the family's
own reads and subjects (`forge.md`, section 9). Wherever a page would
show a part the forge's side does not read, temper's view shows one
paragraph:

> Shown on the forge.

Such a part never shows loading, and never an error.

### 4.3 Where the form is

The form is a page of temper's, `Adopt { project }`, reached from the
navigation's "Adopt a repository" (5.3); its place among a project's
settings is open (section 11). The outcome's sentence (4.7) is what
tells the person a repository is adopted: no page of temper's lists a
project's resources (section 11).

### 4.4 The adoption form

The page's title is "Adopt a repository into PROJECT", PROJECT as `me`
names the project. For a project `me` does not name (before `me`'s
first delivery, or one no longer among the party's projects), it is
"Adopt a repository into project N", N the address's project number,
until `me` names it. The form is the same either way: the engine
judges the adoption, and a refusal at admission (the party's role in
the project) is shown in jig's words (4.6). Its fields are
`temper-engine-domain-forge`'s `Adoption`'s but the repository's id,
which the forge connector locates by owner and name
(`../domain/forge.md`, section 4), and the forge's host, which temper's
server fills from the forge's configuration (section 8). In order:

| Field | Node, named | Default | Rule |
|---|---|---|---|
| forge | text field "Forge number" | `1` | a number from 1 to 65535; one the engine has not configured is refused as no such forge |
| owner | text field "Owner" | empty | required |
| name | text field "Name" | empty | required |
| role | choice group "Role": "Owned: temper pushes, opens pull requests and lands", "Fork: temper contributes through a fork", "Context: temper only reads" | owned | one of the three |
| home | check "Make it the project's home" | checked | a context repository cannot be the home |
| prefix | text field "Branch prefix" | `temper/` | required |
| landing | text field "Landing branch" | empty | required: the forge connector refuses an adoption without one |
| ci | check "It runs CI" | checked | without CI, at least one check |
| checks | text field "Checks: gate numbers, separated by spaces or commas" | empty: none | numbers from 1, each once, at most the client's limit on checks |

Then a button "Adopt", and below it the request's state or outcome.

- **Every field is bounded** by the client's limit on a field's bytes,
  set no larger than the forge connector's name bound in temper's small
  profile (`temper-engine-domain-forge`'s `name_bytes`). A value past
  it is not taken: the field keeps its last value and is marked
  invalid, with the bound in words ("At most N bytes.").
- **Checked before sending:** each rule of the table that the draft
  alone decides (all but whether the forge is configured) is the
  form's own check, never one of what the person may do. A field that
  fails is marked invalid, with one status line under it saying the
  rule in words ("Required.", "A forge number is from 1 to 65535.",
  "Checks are numbers from 1, each once.", "A context repository
  cannot be the project's home.", "Without CI, name at least one
  check."), and nothing is sent. The engine checks everything again.
- **Adopt** makes one keyed request through `jig-client` (4.6), as the
  forge's family's bytes, made once (jig's `client/domain/client.md`,
  section 6). While it is pending the button is disabled and a status
  says its progress as `jig-client` gives it, in jig's words.
- **The forge asked by number, home checked.** `me` names neither the
  configured forges nor whether the project has a home, so the form
  asks for the forge by number and checks home by default; the engine
  refuses a forge it has not configured and a second home, saying
  which. With the forge's project part read, the form chooses among the
  configured forges by name and says whether the project has a home
  (`forge.md`, 5.1).
- **After the outcome,** the outcome stays shown until the person edits
  a field or leaves the page. Adopted, the fields go back to their
  defaults (written by the domain, so the view writes them).
- **Out of date:** when the engine does not serve this client's version
  of the forge's records (jig's `protocol/client.md`, section 9), the
  page says "This temper is out of date for adopting repositories: run
  the temper the engine runs.", the button is disabled, and a pending
  adoption stays in the small store.

### 4.5 Its address and state

- **Address:** `Adopt { project }`, one of temper's addresses (3.5).
- **State:** each field's text and the domain's count of its own
  writes; the pending request's key and progress, at most one; the last
  outcome; the version range the engine serves for the forge's records.
  Bounded by its limits: the field bytes, the checks.
- **Records:** none of its own; its pending adoption is `jig-client`'s
  kept request (4.6).

### 4.6 The records

`temper-client-protocol-wire`, generated from one schema (jig's
`protocol/client.md`, sections 3 and 11):

- **`FORGE`:** the forge connector's number, 1, which is the family's
  owner number on the wire, under which its records travel as bytes
  (jig's `protocol/client.md`, section 11), and the engine's one
  configured connector (`engine.connectors`,
  `../protocol/configuration.md`, section 2).
- **`Owner`:** temper's owners, one enum the engine's root and the
  client root share, with the functions from an owner to its number and
  back (jig's `client/domain/root.md`, section 15): `Forge`, `FORGE`'s
  number; `Pages`, temper's pages, 2; and `Runner`, the agents' tools'
  runner (`tools.md`, section 4), 3: numbers no connector takes, never
  reused. `Pages` and `Runner` ask only for jig's operations, reads and
  subjects, so no request, read or subject of their own reaches the
  engine.
- **`CLIENT_VERSION`:** the version of every record below, 1 at first,
  moved as `../protocol/README.md`, section 9 says. The engine serves
  exactly one version (jig's `protocol/client.md`, section 9).
- **The request, `Adopt`:** the project (u32); the forge's number
  (u16); the owner, the name, the prefix and the landing branch (byte
  strings); the role (owned, fork or context); home and ci (booleans);
  the checks (a list of u32).
- **Its outcome:**
  - *adopted,* with the repository as the forge connector recorded it:
    its forge's number and its id; its owner and name; its role;
    whether it is the project's home; what temper may do there (the
    forge's `Kinds`: read, push, open, land, review, status, comment,
    issue, branch); and whether branch protection was read (unknown,
    absent, or a rule);
  - *refused by the core,* when the core refused the adoption the root
    made on the party's behalf: jig's refusal facts, carried whole as
    `jig-client-protocol-wire` encodes them, and shown in jig's words
    for its refusals;
  - *refused by the forge,* with the forge connector's reason: no such
    forge; not found; no access; refused; in flight; too large;
    unreachable; invalid answer (4.7).

  A refusal at admission (signed out, the party's role in the project,
  a key sent again with another request, a limit) is not an outcome:
  it is jig's answer, which never reaches the connector (jig's
  `client/domain/contract.md`, 3.3), and the form shows it in jig's
  words.
- **The small store's record of a pending adoption** is `jig-client`'s
  kept request, its request encoded by this crate as the family's bytes
  at `CLIENT_VERSION` (jig's `protocol/client.md`, section 10).
- **The forge's documents' records:** its reads, subjects and
  documents, and its other requests, beside `Adopt` (`forge.md`,
  section 9).
- **Bounds:** the schema's ceilings bound every record at both ends,
  and the generated worst cases price each. Within them, the client
  bounds its fields by its own limits (4.4), and the engine's forge
  connector refuses what is past its own as too large (4.7), so a
  record a client can make always decodes.

### 4.7 The outcome in words

| Outcome | The form says |
|---|---|
| adopted | "Adopted OWNER/NAME on forge F as ROLE." then "It is the project's home." if so; "temper may: KINDS." in the forge's words; "Branch protection could not be read." if unknown |
| refused by the core | jig's words for its refusal |
| no such forge | "No forge F is configured." |
| not found | "No repository OWNER/NAME on forge F that temper's forge user can see." |
| no access | "temper's forge user may not read OWNER/NAME on forge F." |
| refused | "The forge connector refused it: another project writes OWNER/NAME, the project has a home already, or branches under PREFIX are not this deployment's." |
| in flight | "OWNER/NAME is being adopted already; try again when that ends." |
| too large | "A field is past the forge's bounds, or the forge's repositories are full." |
| unreachable | "Forge F did not answer; nothing was adopted. Try again." |
| invalid answer | "Forge F answered what temper did not expect; nothing was adopted." |
| unknown past the retention | "It is not known whether OWNER/NAME was adopted." |

KINDS lists what temper may do, in this order and these words: read;
push; open pull requests; land; review; set statuses; comment; create
issues; create and delete branches. The forge connector maps the forge
client's errors to these reasons (section 8).

## 5. The view

### 5.1 Its shape

`temper-client-view` builds temper's pages as skein-ui's tree (skein's
`ui.md`), one owner, temper's:

- **Its state:** skein-ui's view (the last tree, a spare, node ids never
  reused); temper's phrasebook, English, with jig's words for jig's
  enums and a decision's facts (`jig-client`'s `words`) under it,
  overridden phrase by phrase where the mockups word them otherwise
  ("Funds 40 from your pool · 120 left, 80 after", `inbox.html`), and
  temper's own phrase for what `jig-client`'s `words` does not say (an
  executor, an outcome); temper's bindings for the last tree; its
  limits.
- **`build`:** when the domain's shown count moved, it builds the page
  in one outline (skein's `ui.md`, section 14), in the mockups' order:
  - *a banner:* temper's name; the project the person is in, and the
    others to choose from; the link's state, one word for each of
    `jig-client`'s `Link` states ("Signed out", "Connecting" while
    starting, "Live", "Offline", "Out of date"), and "Behind" while a
    page's watch reopens; the person's name and "Sign out";
  - *the navigation* (5.3);
  - *the main:* the page at the address (3.5), its title its level-1
    heading.

  Then skein-ui diffs it and keeps the tree.
- **The pages not about the work** (jig's `client/domain/pages.md`,
  section 8). While the link is signed out or out of date, `me` has
  ended (jig's `client/domain/client.md`, 8.2). The banner is then
  temper's name and the link's state alone, no navigation is offered,
  and the main, in place of the page at the address, is one paragraph
  of temper's words with no heading:
  - *signed out:* "Signed out." (6.6);
  - *out of date,* the engine serving no version of jig's records that
    this client speaks: "This temper is out of date: run the temper the
    engine runs." What is pending stays in the small store (jig's
    `client/domain/client.md`, 8.2).

  Both are phrases of temper's phrasebook: `jig-client`'s `words` has
  none for the link's states, and temper arranges no page of jig's
  (section 1). The page at the address is shown again once the link is
  starting, live or offline. When jig's records are served and the
  forge's are not, the link stays live, and the form says its own out
  of date (4.4).
- **`act`:** a shell's act on a node is resolved by skein-ui in the
  last tree to its binding, and decoded by temper's bindings into the
  domain's action. A node gone, or a token of an older tree, decodes to
  nothing (skein's `ui.md`, section 6).
- **Every node a person acts on has a role and a name,** since skein's
  fake person finds nodes only by them; nothing a person must know is
  said by style alone (skein's `ui.md`, section 8).
- **The look:** the mockups' (`mockups/style.css`, reference material),
  as style tokens each shell interprets or ignores; the terminal prints
  none.

### 5.2 The forge's view

- **Words:** the forge's phrases in temper's phrasebook: the connector,
  resources, patterns and kinds (4.1); "Shown on the forge." (4.2);
  the form's title, field names, roles, rules and button (4.4), its
  title by the project's number included; the outcome (4.7); the form's
  out-of-date sentence (4.4); the navigation entry. Names
  (owners, repositories, branches) are copied in, never translated.
- **The parts:** a connector, resource, pattern or kind is one text
  node of its words; a part the forge's side does not read is the
  paragraph of 4.2; a part it reads is shown as `forge.md` says.
- **The form's page:** a level-1 heading (the title), then the form's
  nodes in the order of 4.4: text fields with their values and write
  counts, the role as a choice group of three choices, home and ci as
  checks, the button, and after them a status for progress and an
  outcome, or an alert for a refusal.
- **Bindings:** writing a field, choosing a role, toggling home or ci,
  pressing Adopt, and going to the page from the navigation entry.
  Typed input is made here: the forge number's digits become a number,
  the checks' words a list, or nothing when they are not one, which the
  form marks invalid.
- **No drawings.**

### 5.3 Navigation

In the mockups' top bar's order: "Inbox", with the count of what needs
the person from `me`; "Chats"; then, for the project the person is in,
"Board" and "Changes"; and after them "Adopt a repository", to
`Adopt { project }`. With no project chosen, the project's entries are
not offered. The mockups' fifth entry, "Tasks", a list of a project's
tasks, is not in the client (section 11). "Adopt a repository" is
offered whatever the person's role: the engine judges it, and a refusal
says why, since the client never computes what a party may do.

### 5.4 Limits

The view's limits are skein-ui's (skein's `ui.md`, section 15), derived
from the domain's: each page's nodes at its windows' most rows, the
forge's form and its outcome's lines, the kinds' list at its longest,
and one paragraph per part not read. Their text across a tree,
`text_bytes`, is the nodes at a node's most text: a lower cap would be
a guess, which skein-ui's builder turns into a panic on the page past
it. skein's text shell counts its worst-case print from these limits,
and reserves for each print only what that page's tree may print
(6.8).

temper's client takes every document within the bounds the engine's
configuration gives its documents. A path's segments have a bound of
their own, 9, apart from the windows' rows (jig's
`client/domain/client.md`, section 14), at least the most the engine
admits in a name (`../protocol/configuration.md`, section 7), so a page
of the forge's work is never refused for a path. A path longer than a
node's text shows its last segments after the mark `…` (as jig's
`client/domain/view.md`, section 7 cuts one), which adds nothing to the
view's limits. An authority's entries have a bound of their own, 12,
apart from the windows' rows (jig's `client/domain/client.md`, section
14), at least the most the engine admits in an authority's list
(`../protocol/configuration.md`, section 7), so a task's page is never
refused for its authority, nor an amendment drafted from part of one.
Each of an authority's lists shows at most the rows per window, then
jig's mark (jig's `client/domain/view.md`, section 7), which adds
nothing to the view's limits.

## 6. The terminal shell

### 6.1 `temper open DIRECTORY LINK_OR_ADDRESS`

- **DIRECTORY** is the client's private directory. The terminal opens
  its parent as a root (skein-shell's `open_root`) and gives its last
  component to `jig-client-shell-native`, whose files make it `0o700` if
  it is absent and refuse it unless it is a directory of this user's
  with no group or other bits (skein's `io.md`, 5.3). It holds the
  bearer credential's file and the small store's, both the native
  shell's (section 7).
- **LINK_OR_ADDRESS** is one of:
  - the link `temper serve` printed,
    `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`: the client
    signs in by it first (section 7);
  - the engine's address, its clients' listener's `http://HOST:PORT`,
    with or without a final `/`: the client signs in with the credential
    DIRECTORY holds. Before its loop, `jig-client-shell-native` reads
    `DIRECTORY/token` once, as it loads the small store, and the host
    gives it to `jig-client` as held (section 7).

  HOST is an IP address, and must be on loopback (in `127.0.0.0/8`, or
  `[::1]`).
- **Exit status:** 0 once the client closed and drained (6.6); 1 when
  it cannot start, with one line on stderr:

  | Cause | stderr |
  |---|---|
  | not two arguments | `temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS` |
  | not an `http://` link or address, or HOST not an IP address and port | `temper open: not a link or an address: ARG` |
  | HOST not on loopback | `temper open: the engine's address must be on loopback` |
  | DIRECTORY refused | `temper open: DIRECTORY: ` and what skein's io found unsafe or failed |
  | an address given and no token in DIRECTORY | `temper open: not signed in: give the link temper serve printed` |
  | the link refused by the engine | `temper open: the link was refused: use the fresh link temper serve printed` |

  A panic aborts the process (fail-stop); the small store and the keys
  make what was pending safe to send again at the next `temper open`.

### 6.2 The loop

- **One host on skein's loop** (skein-shell's `drive`; skein's
  `shell.md`, section 12): temper's client root, its view,
  `jig-client-shell-native`'s transport, and skein's text shell's
  terminal (skein's `ui-text.md`, section 7), with SIGINT and SIGTERM
  from skein-shell's `open_termination_signals`, adopted by io.
- **The text shell's settings** (skein's `ui-text.md`, section 3): the
  name `temper`; the longest line, 4096 bytes; a print interval of one
  second; the status region, the banner's link state (5.1); temper's
  lines, 32 (6.3); and their line bytes, DIRECTORY's bytes and 160:
  6.6's signed out holds DIRECTORY and 84 bytes, and a service's
  credential's line its credential's base64 and at most 57.
- **Time and the seed** are skein-shell's: its clock once per
  iteration, and a seed from the kernel at each start (`seed`), from
  which the client root's children draw their keys' numbers (3.1).
- **Each iteration:** the transport iterates; its loaded records start
  the client (3.4); signing in, by the link or with the token DIRECTORY
  holds (section 7); the token written once signed in by a link; the
  transport's events into the domain; the domain's `fire`; the
  transport's requests carried; the view built; each whole line made
  into one command and done; a print, when one is due; the output
  written.

### 6.3 When it prints

As skein's text shell prints (skein's `ui-text.md`, section 4), at the
interval of 6.2. Before the first tree (signing in), only temper's own
lines, each starting `temper: `.

- **temper's lines wait for room.** The host says each of temper's
  lines only within the room skein's terminal gives (`Terminal::room`).
  A line with none waits in the host, in the order they came; a line
  equal to one already waiting adds none (a sign-in tried again, the
  small store refused again).
- **While a line waits, the person's commands wait.** The host takes no
  command from the terminal while one of temper's lines waits in it, so
  no act of the person's makes a new line; SIGINT and SIGTERM still
  close. What waits is then at most one of each line but a service's
  credential, and one credential for each request already sent, at most
  the request table's requests.
- **Notices are taken whatever the room.** The host takes each of its
  client's notices at every iteration, so a held stdout delays temper's
  lines and the person's commands, never the client: the sign-in's
  retries keep their backoff (6.7). Lines that wait when the terminal
  closes are not printed.

### 6.4 How it prints

As skein's text shell prints (skein's `ui-text.md`, section 5). The
first line is `== TITLE == STATUS`: TITLE the page's level-1 heading,
STATUS the banner's link state, with "Behind" after it while the page
is behind. For example, the adoption page, its banner and navigation
left out:

```
== Adopt a repository into shop == Live
…
[9] Forge number: 1
[10] Owner: acme
[11] Name: (empty) (invalid)
* Required.
Role:
  [12] (*) Owned: temper pushes, opens pull requests and lands
  [13] ( ) Fork: temper contributes through a fork
  [14] ( ) Context: temper only reads
[15] [x] Make it the project's home
```

and so on to `[20] Adopt`.

### 6.5 Numbers and commands

skein's text shell's (skein's `ui-text.md`, section 6). temper adds no
command.

### 6.6 The link, signing out and closing

- **The link's state** is the first line's (6.4). Its change changes
  the tree, so the page is printed again.
- **Signing out** is the banner's "Sign out": the domain asks it, and
  the host gives `jig-client` `SignOut`, which sends jig's sign-out.
  Signed out: the host has the native shell keep none, which erases
  DIRECTORY's token; the main says "Signed out." (5.1), the first line
  reads `== temper == Signed out` (skein's `ui-text.md`, section 5:
  TITLE is "the shell's name when there is none"), and temper prints
  `temper: signed out; to sign in again, run temper open DIRECTORY
  LINK with a link temper serve printed`. A sign-out that did not reach
  the engine leaves the person signed in (jig's
  `client/domain/client.md`, section 12): the banner says "Sign out did
  not reach the engine." and offers "Sign out" again.
- **Another party's kept work:** while the small store holds requests
  of a party other than the one `me` confirms, temper prints `temper:
  work of another party is kept in DIRECTORY, and waits for them` once.
- **Out of date** (5.1): the first line reads `== temper == Out of
  date`, and the main says "This temper is out of date: run the temper
  the engine runs."; temper prints no line of its own. What is pending
  stays in DIRECTORY's small store, closed as below.
- **Kept work dropped** (3.4): when `jig-client` drops kept requests at
  start, temper prints `temper: N requests kept in DIRECTORY were not
  sent: another version of temper made them` once, N the count it told.
- **A service's credential,** when an answer carries one (jig's
  `protocol/client.md`, section 8), is printed once: `temper:
  credential for service N, shown once: CREDENTIAL`, the credential in
  base64url. The client keeps no copy of it.
- **Closing:** `q`, the end of stdin, SIGINT or SIGTERM. The client does
  not sign out. skein's text shell closes (`temper: closed`); no new
  exchange starts; `jig-client-shell-native` finishes the small store's
  last write, then closes its connections and files; the process exits
  0. What is pending stays in DIRECTORY's small store, and the next
  `temper open` on the same DIRECTORY sends it with its keys, within
  the engine's retention.

### 6.7 When the engine is unreachable

- **While signing in by the link,** an exchange with no answer, or
  answered not ready (the engine stopping), prints `temper: signing in:
  no answer from the engine; trying again`, and is made again after a
  backoff of 250 ms, doubling to 8 s. `q` gives up and exits 0. A
  refusal (the link spent or expired, or the engine's sign-ins full)
  exits 1 (6.1); `temper serve` prints a fresh link when a spent one is
  tried (jig's `protocol/client.md`, section 8).
- **Once signed in,** the link is `jig-client`'s to judge (jig's
  `client/domain/client.md`, 8.2). The first line says its state, and
  the page is printed again at each change. Requests made meanwhile are
  kept in the small store and shown as parked; nothing the person did
  is lost, and `q` keeps it in DIRECTORY.
- **A small store that cannot be written** stops that step's requests,
  whose cards say they were not sent (`NotStored`), and prints `temper:
  cannot write the small store in DIRECTORY`.

### 6.8 Pure where it decides

The renderer and the command reader are skein's text shell's, step code
under the subset, bounded by its `worst_case` (skein's `ui-text.md`,
section 3). The host adds the terminal's worst case
(`Terminal::worst_case`) to its memory, and the terminal's operations
(`Terminal::operations`) to its ring's, beside its client's and its
own; it copies none of the terminal's count. temper's view limits put
that worst case past a u32 of bytes, and the terminal starts from them,
as `temper open` makes it, in a unit test (section 10). What temper
decides is in its domain and its view, both step code; golden text
fixes how temper's pages print (section 10).

### 6.9 While a run's text streams

The terminal is chiefly the real loop's test surface (`../testing.md`,
2.4): people use the browser, or a later full-screen terminal. While a
run's text streams, the terminal reprints the whole page at each print
interval (6.3), as skein's text shell does, with the chat's provisional
turn as the page draws it (section 3.5); the print interval is the only
pace it keeps. No rule of skein's text shell is added for streaming.
A command typed while text streams is done as at any time (6.5).

## 7. Signing in and the small store

How jig signs a client in, and how a native client keeps its credential
and its small store, are jig's (jig's `protocol/client.md`, sections 8,
10 and 12; jig's `client/domain/client.md`, sections 11 and 12).
temper's:

- **The printed link** (jig's `protocol/client.md`, section 8): at
  start, `temper serve` prints a sign-in link for the deployment's
  first owner (`../protocol/README.md`, section 4), in the default
  mapping's link route under temper's prefix,
  `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`. `temper open
  DIRECTORY LINK` checks that the link names the engine's address on
  loopback and that route, and gives the secret to `jig-client`
  (`SignIn { with: Link(secret) }`). The secret is in the command line
  only until the sign-in spends it; the client root passes it on and
  keeps nothing of it.
- **The bearer credential,** the sign-in's number beside 32 random
  bytes, is `DIRECTORY/token`, a private file (`0o600`), which
  `jig-client-shell-native` writes where the host says, when it says.
  Once signed in by a link, the host reads the credential from
  `jig-client`'s `link()`, through the client root, and has the native
  shell keep it, before anything is told of the sign-in; a new sign-in
  by a link has it written anew; signed out, the host has the native
  shell keep none, which erases it. A later `temper open DIRECTORY
  ADDRESS` has the native shell read it before its loop (6.1), and the
  host gives it back as held (`SignIn { with: Held(credential) }`). It
  is never printed and never an argument, and neither temper's root
  nor its view keeps it.
- **The small store** is `DIRECTORY/store`, `jig-client-shell-native`'s
  file, holding `jig-client`'s records alone: the requests it keeps,
  jig's and the forge's adoptions, pending. temper's pages keep none
  (3.4).
- **Loopback only** in standalone (`../protocol/configuration.md`,
  section 2; 6.1), so a bearer token never crosses a network in
  plaintext. Clients between hosts, over TLS, come after standalone.
- **One person.** The deployment's link signs in its first owner;
  signing in through the forge is open (section 11).

## 8. On the engine's side

The engine's service is `../protocol/README.md`'s (section 6). What it
does for the client:

- **The clients' listener** is `temper-service`'s: TCP on the loopback
  address temper's configuration names
  (`../protocol/configuration.md`, section 3). On it, it mounts jig's
  clients' component (`jig-protocol-engine-clients`) with jig's default
  HTTP mapping's engine end (`jig-protocol-engine-clients-http`) under
  the prefix `/jig` (jig's `protocol/service.md`, section 2). Every
  other path is answered not found, the browser's page and bundle aside
  (`browser.md`).
- **The printed link opened in a browser:** a `GET` of the link's route
  spends nothing, and the mapping leaves it to the application's page
  (jig's `protocol/browser.md`, section 11). Where temper serves no
  page, its server answers it not found, with `Referrer-Policy:
  no-referrer`; the link is `temper open`'s (6.1).
- **jig's example guards, as they are** (jig's `protocol/browser.md`,
  12.2), the Host allow-list holding the listener's own address. The
  terminal sends its credential as a bearer token; the cookie is the
  browser's (`browser.md`).
- **jig's clients' component** (jig's `protocol/client.md`) serves jig's
  wire's versions and, for the family `FORGE`, `CLIENT_VERSION` alone
  (4.6). temper gives it no `Mint` in standalone: the printed link and
  services' credentials are jig's own ways of signing in (jig's
  `protocol/client.md`, section 8).
- **The forge's request, decoded before the root** (jig's
  `protocol/client.md`, section 11). An application request of the
  family `FORGE` at `CLIENT_VERSION` is decoded by
  `temper-client-protocol-wire`; one that does not decode, or of
  another family or version, is answered undecodable, as jig's
  reference service answers it. The routing fills the adoption's host
  from that forge's configured `git` base: its host, with `:port` when
  it names one, as the forge's resource paths have it
  (`../protocol/configuration.md`, section 3). It leaves the host empty
  for a forge number not configured, and hands the root the party's
  keyed request: its reply, its sign-in, its key, the digest of the
  record's bytes, and the adoption.
- **The root routes it** (`../domain/forge.md`, section 4;
  `../domain/root.md`, 5.4): jig's people child admits it by key, and
  refuses at admission as jig's answer; admitted, it goes to the forge
  connector, which refuses a forge not configured (no such forge),
  locates the repository by owner and name (not found when the forge
  has none), and makes jig's adoption request on the party's behalf
  (`adopt_repository_ask`), which the core judges as the party's own.
  The forge connector's adoption then reads what temper may do there.
  The outcome is kept under the request's key for the retention and
  answered once durable (jig's `client/domain/contract.md`, 3.3).
- **The forge's errors as reasons,** by the forge connector
  (`temper-engine-domain-forge`'s `reason`, from `skein-forge`'s
  `Error`, skein's `forge.md`, section 3.4): `Missing` is not found;
  `Forbidden` is no access; `Refused` is refused; `Busy` is in flight;
  `TooLarge` and `Full` are too large; `Unavailable`, `Timeout` and
  `RateLimited` are unreachable; any other is an invalid answer.
- **The outcome, encoded** by `temper-client-protocol-wire`, goes back
  as the family's answer to jig's clients' component.
- **The forge's reads and watches** are decoded by
  `temper-client-protocol-wire` the same way and go to the root
  (`../domain/root.md`, 5.3; `forge.md`, section 9). One the wire has
  no record for does not decode: the routing answers the read with no
  record (unknown) and refuses the watch as unknown, without the root,
  as jig's reference service answers an undecodable one.

## 10. Testing

The tiers, the fakes and the budgets are `../testing.md`'s (sections 2,
4 and 6.1); the client's joint world is its 2.1 and 6.3. The client's
own tests:

- **Step tests:** the domain's pages (what each watches and reads as it
  opens and closes, each act as the operations it sends, the
  confirmation's rule of 3.6, routing by asker); the forge side's steps
  (the form's rules, its request, every outcome, the version check);
  the view (each page from fixture states, read by skein's fake
  person's queries; the forge's words for every path shape and every
  kind, the form in each state, every outcome);
  `temper-client-protocol-wire` (golden bytes, bounds and fuzz targets,
  as skein's `codec.md`, section 6 has them); and the engine's routing
  of the forge's family both ways.
- **Golden text:** temper's pages as skein's text shell prints them,
  kept as files beside the tests and changed only by hand: the inbox
  with entries of each kind, the chats, a chat with its conversation and
  a proposal in place, a board, a goal's page with its plan, an agent's
  task, the changes, the adoption form in each state of 4.4 and 4.7, and
  signed out.
- **The terminal's machine world,** in `temper-client-shell-terminal`'s
  own tests (`../testing.md`, section 7): the host against a scripted
  stdin and stdout and a scripted transport: signing in by the link and
  by a kept token, a page printed after an act, the end of stdin, an
  interrupt, `q` drained. And, in a unit test, the terminal made from
  `limits::native()`'s limits, as `temper open` makes it, so the gate
  meets a refusal of temper's own limits before the process tier does.
- **`tests/client`** (`temper-client-world`), the joint world
  (`../testing.md`, 2.1 and 6.3): temper's client root and view, with
  skein's fake person acting on the tree, joined to temper's engine
  root through jig's clients' component, with no transport between,
  with jig's client worlds' faults and jig's check (jig's
  `client/domain/testing.md`, sections 5, 6 and 7). The forge's request
  and outcome are decoded and encoded by the engine's routing (section
  8), exhaustive both ways. Stories: signing in and seeing the inbox; a
  chat started and answered; a goal proposed in a chat, accepted in the
  inbox, and gone from the chat's card; goals reordered on the board; a
  held task released from a goal's plan; a repository adopted from the
  navigation's link, its outcome read and the engine holding it;
  refused: not found, a second home, a field the form refuses; a part
  of the forge's saying "Shown on the forge."; with jig's faults (drops,
  the engine restarting, a double press, a reload with an adoption
  pending, kept work), each request made once. jig's check holds in
  every story, said in temper's phrasebook, which temper's world gives
  jig's referee (jig's `client/domain/testing.md`, 6.2).
- **The one-process world** (`../testing.md`, 2.5): skein's fake person
  in front of temper's terminal client (its client root, view and
  transport), joined through bytes to `temper-service` on skein's
  simulator: the same stories through jig's default HTTP mapping and
  `temper-client-protocol-wire`, killed and restarted with an adoption
  pending.
- **The real loop's terminal** (`../testing.md`, 2.4): temper's client
  root and view under skein's text shell, over jig's native transport,
  in the same loop as the service, driven by a typist that types
  commands and reads printed pages, with a fake LLM, forge and git. Its
  live-text story: a reply held after its first piece is read printed
  under "Writing…", then replaced by the committed turn.
- **The process tier** (`../testing.md`, 2.6): the real `temper`
  binary; skein's fake person through temper's client signs in by the
  printed link and adopts a repository on an owned Forgejo
  (`../protocol/setup.md`, section 2), with "It runs CI" checked, since
  its runner runs Actions. And `temper open` itself, stdin and stdout
  piped: it signs in by the printed link, prints the inbox, adopts a
  repository by commands, and exits 0 on `q` after a drained close. And
  jig's attack story that needs no browser, a rebound Host, against
  `temper serve`'s clients' listener: a request naming a host off the
  list is answered `421`, with nothing done (jig's
  `protocol/browser.md`, section 15). The cross-site request and the
  framed page need a browser, and come with temper's browser tier
  (`browser.md`, section 4).

## 11. Open questions

- **Pages the mockups do not show:** a project's page with its
  resources, findings, notes and settings (policy, people and roles,
  spend; jig's `client/domain/pages.md`, section 6, and the forge's
  parts in `forge.md`, 5.3 and 7.3); the deployment's page; the "Tasks"
  list of a project; a procedure's task and a person task beyond a
  tracked task's header and plan; making a service and showing its
  credential once; another party's kept work. Default: not in temper's
  client; the person acts on them through the engine's configuration,
  or not at all.
- **The form's place:** among a project's settings, once temper has
  that page (`forge.md`, 5.3). Default: a page of its own, from the
  navigation.
- **The forge's `Refused`, split** into its causes (another project
  writes the repository; the project has a home; another deployment's
  branches under the prefix), so the form names the one. Default: the
  sentence of 4.7 naming the three.
- **The forge as people's sign-in provider:** temper's own routes on its
  clients' listener run the forge's OAuth flow, read who the person is,
  and ask jig's clients' component to mint a credential for that
  identity (`Mint`, with the forge's provider number, a provider of
  kind application in jig's `providers` member, the person's id there,
  their login and name; jig's `protocol/client.md`, section 8).
  A browser then holds it as the default mapping's cookie; a terminal
  needs a flow of its own, such as a device code. Default: the
  deployment's link only.
- **What a page keeps across a restart:** where the client was, the
  composers' words and the form's values, as records of temper's own,
  which the terminal keeps beside `jig-client`'s small store, never in
  it (jig's `protocol/client.md`, section 10). Default: none; the client
  opens at the inbox.
- **Clients that are not a person's:** a command run once and a
  service acting as a party are the agents' tools, `temper ctl`
  (`../tools.md`). A bot (`../draft/factory.md`): none, until the
  factory's lab or run side needs one.
- **Several terminals:** each `temper open` with its own DIRECTORY is a
  client of its own. Two on one DIRECTORY share its token and write its
  small store whole over each other. Default: not supported and not
  detected; a lock on DIRECTORY is the candidate, which `temper ctl`
  takes for its own calls (`../tools.md`, 8.3).
- **A fresh link from the client:** default: the person runs `temper
  open` with the last link, whose refusal makes `temper serve` print a
  fresh one.
- **The person's time zone:** default universal time (an offset of 0)
  for every time the view says.
