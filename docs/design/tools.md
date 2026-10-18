# Tools: temper for agents

Provisional, 2026-10-17, revised 2026-10-18. How agents drive temper:
they read what it holds, act through jig's operations and wait for what
changes, with `temper ctl`, a command line. It serves the agents
(smith's) that QA and develop temper's core: they drive a throwaway
deployment on coding tasks of growing size, then on temper's features,
until the first milestone (`README.md`, section 2) is usable for real
tasks. Their QA loop extends the tools area by area (5.5). What
the tools ask of jig is section 13; an MCP endpoint and the tools'
extraction into jig are later (section 14). What is still open is
listed in section 15.

## 1. In one page

- **Agents drive the plumbing, not the pages.** The tools are jig's
  reads, watches and operations (jig's `client/domain/contract.md`),
  jig's helpers where they fit (`shapes`), and temper's forge family.
  An agent never reads a page, a position on a screen or a layout
  (section 3).
- **Each result says what can be done next.** jig's documents carry the
  acts the party may make (made, proposable, refused); every result
  prints them as the calls that make them, with the revisions they name
  (section 6).
- **One table of tools.** Each tool is defined once: its name, its
  arguments, its result, its line of help. The parser, the help, the
  JSON and the agents' guide are derived from it (5.1).
- **Waiting is a tool.** `wait` blocks on a watch until a subject
  changes from what the agent saw, or meets a condition, or a timeout
  passes (5.4).
- **Each request once.** A tool that makes something requires a key the
  agent names; asked again under that name, it gets the first answer,
  never a second chat. Acts that decide name the revision the agent saw
  (section 7). The named key is asked of jig (section 13).
- **A turn too large for a page is read alone,** by jig's read of one
  turn (5.2, section 13).
- **Thin command line.** `temper ctl DIRECTORY TOOL ...` makes one call
  per process, in `jig-client`'s one-shot mode, over jig's native
  transport, and prints its result (section 8).
- **The deployment under test is a box** (`protocol/setup.md`, section
  7): the agent's harness starts it, signs in by its printed link, and
  throws it away (section 10).
- **Grown by areas,** the first designed in full, the others named
  (5.5); **later, named:** the MCP endpoint and the extraction into
  `jig-client-tools` (section 14).

## 2. What it is for, and what not

- **The first QA loop.** Agents use temper to do coding work of growing
  size, through growing parts of its surface: a chat that changes one
  file; then larger changes, held changes released, questions answered;
  then goals with plans, decisions, several parties. What breaks is
  fixed, and what is slow made faster, each fix with a story in the
  worlds (11.6).
- **Production later.** An agent driving temper for a person, or a
  service of a project acting as a party, uses the same tools; MCP gives
  them to agents that have no shell (section 14).
- **Not the person's pages.** temper's pages are the person's, proven by
  the fake person's stories in `tests/client` (`client.md`, section 10).
  No tool waits for a page: a tool reads jig's documents, so no page is
  a tool's prerequisite.
- **Not the browser.** Once temper has one, its QA is agents driving the
  browser as a person does. These tools are no way around it.
- **Not the deployment.** Starting temper with a forge, a runner and a
  model is `protocol/setup.md`'s; resetting it is the harness's, never a
  tool's (section 10).

## 3. Why the plumbing, not the pages

- **The pages are for people.** The text view's numbers are positions
  that change between prints, the banner and navigation cost an agent's
  context on every read, and a page shows only what its mockup chose. So
  the text view is no agent's driver; these tools are, for temper
  (`draft/qa.md`, 4.2).
- **jig's client was made for this.** It is presentation-agnostic: "a
  browser, a terminal, a chat bot, a one-shot command and a service with
  no presentation hold the same client" (jig's `client/domain/client.md`,
  section 1). Its documents are typed, with stable numbers, revisions
  and offered acts (jig's `client/domain/contract.md`, 5.1 to 5.3).
- **Offered acts replace discovery.** An agent learns what it may do
  from the document, as a person learns it from buttons: each act made,
  proposable (to whom) or refused (why), with the bounds a draft is
  compared with.
- **The engine judges again.** Whatever an agent sends, the engine
  checks as it checks a person's request: its role, its pool, the
  revision it names. The tools add no authority and remove no check.

## 4. The crates

```
temper-client-tools          the table, a call, the runner, marks and conditions, the rendering (text,
│                            JSON), the argv parser and help, the guide; the tools' root
├── jig-client               jig's client and its helpers (shapes, facts, thread, words)
├── temper-client-domain     the forge's client side (client.md, 3.1 and section 4) and its words (4.1, 4.7)
├── temper-client-protocol-wire  the forge family's records (client.md, 4.6)
└── skein-json               the JSON writer and tokenizer
temper-client-shell-cli      the host: argv to a call, the one-shot host over jig-client-shell-native,
│                            the directory's lock, printing, exit status
├── temper-client-tools
└── jig-client-shell-native  jig's native transport: HTTP, the small store's file, the token's
temper                       the binary: `temper ctl` runs the host
```

- **Named by the client naming rule** (jig's `README.md`, section 15):
  the tools are to agents what `temper-client-view` is to people.
- **jig's part apart from temper's.** `temper-client-tools`' module
  `jig` holds what is generic: jig's part of the table, the call, the
  runner, marks and conditions, the rendering. It names no temper item:
  a boundary test fails the gate when a file under `src/jig/` names
  `temper`, or an item of the crate outside `crate::jig`. Extraction is then
  that module moved into a crate of jig's (section 14).
- **The forge's side is reused, not copied.** The tools' root holds the
  forge's client side of `temper-client-domain`, its adoption's ask,
  owner and words, and none of its pages.
- **The tools' root** is a client root in jig's one shape (jig's
  `client/domain/root.md`, section 3): `jig-client` held; two children,
  each an owner and an asker, the runner and the forge's side; routing
  by asker. It holds no page and no view, as a service's root holds
  none (jig's `client/domain/root.md`, section 11). Each call is the
  runner's asker of its own, so several calls in flight need nothing
  new of it.
- **Words:** jig's for jig's enums (`jig-client`'s `words`) and the
  forge's of `client.md`, 4.1 and 4.7, so an agent and a person name
  things alike. The view's overrides for its mockups stay the view's.
- **Step code under the subset:** the runner, the rendering, the table,
  the tools' root. **Ordinary Rust:** the host.

## 5. The tools

### 5.1 One table

- **A tool is a record of the table:** its name; a line saying what it
  does; its arguments, each a name, a type (a number, a word from a set,
  a text, a file, a duration), whether it is required, and a line; its
  result, a typed record; whether it is keyed (section 7); its area
  (5.5).
- **Everything else is derived from it:** the parser and `temper ctl
  help [TOOL]`, the JSON of arguments and results, and the tool names
  the guide may use (11.5). A test checks that each derivation names
  every tool.
- **A call is a value:** a tool's name and its arguments, decoded from
  argv or from JSON and checked against the table before anything is
  sent. The runner takes a call and gives back its result; the host
  interprets neither.
- **Projects by name or number;** a name is resolved through `me`.

### 5.2 Reads

A read answers with a document, or a page and its cursor, and its mark
(5.4). It needs no key.

| Tool | What it answers | jig's |
|---|---|---|
| `me` | the party, its projects with their roles and pools, what waits for it counted, the bounds of what it may create, the retention | `me` |
| `inbox [--project P] [--kind K] [--root N] [--after C]` | what needs the party, then updates, each with its acts | `inbox` |
| `tasks --project P [--label chat\|goal] [--tracked] [--mine] [--ended] [--order activity\|priority] [--after C]` | task lines | `tasks{query}` (jig's `client/domain/contract.md`, 6.1) |
| `task N [--tree] [--activity K] [--run]` | a task's document, with its acts; with its live tree, its K latest activity lines, its live run, in one answer | `task{n}`, `tree{root}`, `activity{n}`, `run{n}` |
| `chat N [--before C]` | a conversation: turns, words and provisional text in order, a page from its end | `activity{n}`, `run{n}`, through `thread` |
| `activity N [--before C] [--limit K]` | a task's activity, a page from its end | `activity{n}` |
| `turn N A.T [--part K] [--from B]` | one turn of task N, attempt A, turn T, whole: its text, each call and its result | jig's read of one turn (section 13) |
| `run N` | a task's live run: its attempt, its provisional text, its current call | `run{n}` |

- **A turn too large for a page** comes in a page by its size in place
  of its bytes (jig's `protocol/store.md`, 7.3). `activity`, `chat` and
  `task --activity` print it as one line, its number, its size and the
  call that reads it: `turn 3.7 · 412 KB · read with: turn 41 3.7`.
- **`turn`** prints each part of the turn cut to the tools' window, each
  cut with the call that reads on (`--part K --from B`). A turn past the
  client's limit for one document is answered with its size and that
  it is past it; the agent reports it as a finding.
- **The forge's changes are read on the forge,** with its API or `git`,
  as a person reads them there. A part of the forge's that the engine
  does not serve prints "Shown on the forge.", with the resource in the
  forge's words (`client.md`, 4.1 and 4.2). The forge's read family
  (`web/ux/forge.md`, section 9) joins the table as tools of its own
  (section 14).

### 5.3 Acts

Each act is one keyed request through `jig-client`'s request table,
answered once durable (section 7). `K` marks the tools whose key the
agent must name.

| Tool | What it does | jig's | K |
|---|---|---|---|
| `chat --project P TEXT` | starts a chat | `shapes::chat`, a `Create` | yes |
| `message N TEXT` | words to a task | `Message` | yes |
| `close N [--move D]...` | closes a chat: its live delegates cancelled, or moved to the party where named | `shapes::close_chat`, a compound request | yes |
| `adopt --project P --forge F --owner O --name N [--role owned\|fork\|context] [--home]` | adopts a repository | the forge family's request (`client.md`, 4.4) | yes |
| `answer N M TEXT` | answers question M of task N | `Answer` | |
| `decide P/K accept\|reject\|pass-up [REASON]` | decides a proposal | `Decide` | |
| `decide N --hold R release\|leave\|pass-up [REASON]` | decides an escalation | `Decide` | |
| `release N --hold R [--again]` | lifts a hold | `Release` | |
| `stop N` | cancels a task's run, the task held | `Stop` | |
| `cancel N [--grace D] REASON` | cancels a task and its tree | `Cancel` | |

- **The arguments are jig's,** named as its operations name them, with
  the revisions they need (jig's `client/domain/contract.md`, 5.2): a
  proposal by its number, an escalation and a release by the hold's
  revision. An agent copies them from the acts a result printed (6.1);
  it never has to know which an act needs.
- **No confirmation.** The person's client says what an act will do
  before sending it (`client.md`, 3.6); an agent reads it in the
  document's acts and their effects, and the revision is its promise.
- **A draft beyond the party's bounds** is compared with the bounds the
  document gives, as the person's client does (jig's
  `client/domain/contract.md`, 5.3): within, sent; between the bounds,
  sent as `Propose` where it is proposable, and the result says so; past
  them, refused by the client, nothing sent. Where `me` gives no bound
  for a draft, it is sent and the engine's answer printed.

### 5.4 Waiting

```
wait SUBJECT [--since MARK] [--until CONDITION] [--timeout DURATION]
```

- **The subject** is a watch's (jig's `client/domain/contract.md`, 7.1):
  `me`, `inbox`, `task N`, `tree N`, `run N`, or a task query as `tasks`
  takes it.
- **A mark** is a digest of the subject's document as the engine
  delivered it, the first 8 bytes of its SHA-256 (skein's `lib.md`,
  6.2), printed as 16 hex digits with every read of that subject and
  every wait's answer. jig's revisions move only with what a party
  decides about (jig's `client/domain/contract.md`, 5.2), so they cannot
  say that a document changed; the mark can, and needs nothing of jig.
- **`--since MARK`:** answered at the first delivery whose mark differs;
  without it, at the first change after the watch's first delivery.
- **`--until CONDITION`:** answered at the first delivery that meets it,
  the first delivery included. Conditions are named facts over the
  document:

  | Subject | Conditions |
  |---|---|
  | `me` | `waiting` (something waits for the party) |
  | `inbox` | `waiting`; `updates` (an unread update) |
  | `task N` | `ended`; `waiting-on-me`; `held`; `phase:<phase>` |
  | `tree N` | `waiting-on-me` (a proposal or a question below waits for the party); `ended` |
  | `run N` | `ended` |

- **`--timeout`:** 300 seconds by default, at most 3600; answered with
  the current document, its mark, and that the condition is not met
  (6.3).
- **Level-triggered, as the watch is** (jig's
  `client/domain/contract.md`, 7.2): deliveries between are coalesced,
  and the answer is the document as it is when answered. A change before
  the wait began is in its first delivery, which `--since` compares.
- **The loop:** `wait me --until waiting`; `inbox`; act; again. An agent
  following one chat: `wait tree N --until waiting-on-me`, or `wait task
  N --until ended`.
- **On a box,** the forge's webhooks make the engine's documents move
  soon after the forge does; with polling alone, the same waits answer
  later (`protocol/setup.md`, section 10).

### 5.5 Areas

The table grows in the order the QA loop reaches temper's surface. Each
area lands with its stories (section 11).

1. **A chat to a landed change** (the first QA loop, the milestone's
   story, `README.md`, section 2): `sign-in`, `sign-out`, `pending`,
   `help`; every read of 5.2; `wait`; every act of 5.3.
2. **Goals and plans:** `goal --project P [--priority I] [--budget A |
   --as-allowed] [--deadline T] TEXT` (`shapes::set_goal`, keyed),
   `proposal P/K`, `amend N --spec R CHANGE... REASON`, `withdraw P/K`,
   `seen`.
3. **Several parties:** `service --project P --role R --out FILE`
   (jig's `MakeService`, keyed; section 9), `sign-in --service FILE`,
   `project P`, `deployment`, `move N REASON`, a person task's `take`,
   `hand-back` and `answer N --rev R`, and the project's own operations,
   `policy`, `role`, `pool` (jig's `domain/people.md`, 5.2).
4. **jig's plumbing whole:** `create`, `propose`, `compound --project P
   FILE`, a batch, a proposal or a compound request as JSON (keyed).

- **Area 1 is designed here in full.** Areas 2 to 4 name their tools and
  jig's operations; each area's arguments are checked against jig's
  operations as they stand when the QA loop reaches it, by a design
  pass (`development.md`, section 4), and land with it.
- **The QA loop may ask for a tool no area names.** It goes to the
  designer as a design question, and joins the table as a row of an
  area.

## 6. What a result looks like

### 6.1 Text

The default: plain UTF-8 lines, no colour, no wrapping. A document
prints its header line, its fields, its acts and its mark. A goal
waiting on the agent:

```
task 41 · goal · running · shop · priority 2
asked: Add a /health endpoint that reports the store's state
requester: you · spent 1.20 of 5.00 · no deadline
waiting on: you: question 3 of task 44, "Which port should it listen on?"
plan: 4 live (2 running, 1 waiting, 1 held below)
acts:
  answer 44 3 TEXT
  stop 41
  cancel 41 REASON
mark: task 7c1e09a2b44f0d13
```

- **Acts as calls,** without the `temper ctl DIRECTORY` prefix; a keyed
  act's call ends with `--key NAME`. A proposable act says to whom; a
  refused one says why, and prints no call. The host gives the
  rendering how a call is printed.
- **jig's words and the forge's** (section 4): phases, holds, what a
  task waits on and refusals in the words a person reads.
- **Lists** print one line per row, its number first, then the cursor
  for the next page, or that there is none.
- **An act's outcome** prints what was made or decided and its numbers:
  `done: task 41, chat`; `proposed: 3/17, to maintainers`; `decided
  first: by ada, accepted, 14:02`; for `adopt`, `client.md`, 4.7's
  words.

### 6.2 JSON

`--json` prints the result record instead: one JSON document on stdout,
written with `skein-json`'s writer, the result's fields by their names
in the table, numbers as jig's integers, no floats. The JSON is the
contract for scripts; the text follows temper's words, and may change
with them.

### 6.3 Exit status

| Status | Meaning | Prints |
|---|---|---|
| 0 | a read answered, an act done or proposed, a wait's condition met | the result |
| 1 | the call failed: usage, the directory refused, the engine unreachable, signed out, the client's limit, unknown past the retention | one line on stderr, `temper ctl: ...` |
| 2 | refused by the engine | jig's refusal, typed in JSON, in words in text; the act's alternatives where the refusal names them (read again, propose instead) |
| 3 | a wait's timeout, its condition not met | the current document and its mark |

### 6.4 Bounded

- **Every result has a ceiling,** in rows and bytes: a list's page, the
  activity lines `task` adds, a conversation's page, a turn's part. A
  text cut prints how much was left and the call that reads the rest.
- **Long texts are paged, never whole by default:** `chat`, `activity`
  and `turn` take a cursor or an offset. An agent's context is the
  budget the defaults are chosen for.
- **The numbers** are constants of `temper-client-tools`' `limits`
  module, beside the client's (`client.md`, section 2), and within
  them. A path is never cut: the tools print it whole, within the
  client's bound on a path's segments (`jig-client-domain`'s `Limits`,
  `segments`).

## 7. Requests: once, and named

- **Through jig's request table.** Every act is a keyed request:
  `jig-client` keeps it before sending it, sends it again under its key
  until its answer is durable, and answers it once (jig's
  `client/domain/client.md`, section 6).
- **The agent names the key** of every tool marked `K`: `--key NAME`, a
  word of 1 to 64 bytes of `a` to `z`, `0` to `9` and `-`, such as
  `health-chat`. A call without it is refused as a usage error, before
  anything is sent.
- **A named key is the engine's key:** the first 16 bytes of the SHA-256
  of `temper-tools`, a zero byte and the name. jig scopes it to the party
  (jig's `client/domain/contract.md`, 3.1). The same name asked again by
  the same party, within the engine's retention, is the same request: a
  rerun after a lost answer, an agent's own timeout or a lost context
  gets the first answer, never a second chat. The same name with another
  ask is refused as a key conflict. Nothing is kept on the agent's side,
  so a rerun is the same request from any directory signed in as the
  party.
- **Names are the party's.** Agents sharing a party give their names a
  prefix of their own; the guide says so (11.5).
- **Acts that decide name the revision the agent saw;** a moved one is
  refused, and the agent reads again. A repeat of a decision is answered
  as decided first, by the party itself.
- **The other acts** (`stop`, `cancel`, `decide`, ...) take `--key`
  optionally; without it the table draws one, which the small store
  keeps until answered (8.2).

## 8. The command line: `temper ctl`

### 8.1 Its shape

```
temper ctl DIRECTORY TOOL [ARGUMENT...] [--json]
temper ctl DIRECTORY --call FILE [--json]
temper ctl help [TOOL]
```

- **DIRECTORY** is the client's private directory, as for `temper open`
  (`client.md`, 6.1): its credential, `token`, its small store, `store`,
  and the engine's address, `address`, all `jig-client-shell-native`'s
  or beside them, `0o700` and refused unless the user's own. A
  DIRECTORY is `temper ctl`'s or `temper open`'s, never both's (section 15).
- **`--call FILE`** reads one call as JSON, `{"tool": ..., "arguments":
  {...}}`, from FILE, or from stdin for `-`.
- **`sign-in LINK`** signs in by the link `temper serve` printed, on
  loopback, writes the token and the address the link names, as `temper
  open` does (`client.md`, section 7). `sign-out` ends the sign-in and
  erases the token. Every other tool signs in with the token DIRECTORY
  holds (`SignIn { with: Held }`).
- **Loopback only,** as `temper open`, until the clients' listener
  serves TLS.
- **No configuration file;** its inputs are its arguments and DIRECTORY.

### 8.2 One call, one process

The host of `jig-client`'s one-shot mode (jig's
`client/domain/client.md`, section 20), ordinary Rust on skein's loop,
as the terminal's host is:

1. parse the arguments against the table; a usage error exits 1 before
   anything is opened;
2. open DIRECTORY; for a call that sends, take its lock (8.3) and load
   the small store's records;
3. start the tools' root, sign in with the token, wait for `me`;
4. send what the store kept, each answered as a line on stderr, `kept:
   ...`;
5. run the call: a read answered, an act's answer durable, a wait's
   condition met or its timeout;
6. print the result on stdout, close the stream, exit with the call's
   status.

- **A termination signal** stops the call; what it sent stays in the
  small store, and the next call that sends sends it again under its
  key.
- **`pending`** lists what the small store keeps, and its progress.

### 8.3 Several calls at once

An agent runs a `wait` while it acts, or several agents share a party.

- **A call that sends** takes an exclusive lock on DIRECTORY's store for
  its run, so two never write the small store over each other. A second
  waits for the lock.
- **A read and a wait** take no lock and start the client with nothing
  kept: they write no record and send nothing but reads and watches.
- **Several parties** are several directories, each signed in as its
  own.

## 9. Who the agent is

- **Any credential.** An agent acting for a person may hold the person's
  credential or one of its own: the tools, their calls and their
  results are the same either way, and jig judges each request as the
  party whose credential it is (jig's `domain/people.md`, section 3).
- **The person's own:** the printed link's sign-in. The QA loop's first
  identity is the deployment's first owner.
- **Its own, a service's** (area 3): an owner makes a service in a
  project, with a role (`service`), and jig's clients' component answers
  its credential once; the tool writes it to the file `--out` names,
  `0o600`, refused if the file exists, and never prints it. The agent
  signs in with it (`sign-in --service FILE`) and acts within that
  role's authority and pool. QA plays several parties this way.
- **Credentials are files,** private, never arguments, never printed.
- **The agent's LLM account is not the tools'.** The deployment's model
  account is handed in by its configuration, as the owned example has
  it (`protocol/setup.md`, 8.1; `protocol/credentials.md`, section 3).

## 10. The box

The deployment an agent drives is an owned setup run by `temper box`
(`protocol/setup.md`, section 7), in a scratch directory of the QA
harness's:

- **Started:** `temper box CONFIG.json`; its stdout carries `temper
  serve`'s sign-in links, and the harness passes the first to `temper
  ctl DIRECTORY sign-in LINK`.
- **A repository to adopt,** in an org of the owned Forgejo, adopted
  with `adopt` as the first act of area 1.
- **The forge read directly:** its API and `git` on `owned.listen`, as
  the person (`protocol/setup.md`, 6.5). A token for those reads is
  `protocol/setup.md`'s open question (section 16).
- **Logs:** the engine's and the worker's in the box's stderr, each line
  prefixed by its step; Forgejo's in its directory. Outside the tools.
- **Thrown away** by stopping the box with a termination signal and
  removing its directory: the harness's, never a tool's.

## 11. Testing

### 11.1 Step tests

- **The table:** each tool's argv parsed into its call and back, and its
  JSON; usage errors; the help, as golden text.
- **The rendering:** each result kind in text and JSON, as golden text,
  with jig's and the forge's words; a turn by its size; a cut and its
  call.
- **Marks and conditions:** a mark moves with a document's change and
  only then; each condition over documents that meet and miss it.
- **The boundary:** `src/jig/` names no temper item (section 4).

### 11.2 `tests/client`

`temper-client-world` runs the tools' root against temper's engine root,
beside the fake person, as a scripted agent making calls (`client.md`,
section 10; `testing.md`, 6.3). Area 1's stories: a chat started, its
reply waited for, a message, a turn too large for a page read whole,
the chat closed; a repository adopted; a held task released. The
world's faults hold: a call cut by a link drop or an engine restart,
asked again under its named key, is made once. Each later area adds its
stories.

### 11.3 The real loop

`tests/real-loop` hosts the tools' root over jig's native transport,
the standalone engine and worker, smith's agents and skein's fake LLM
peer on one thread and one real ring (`testing.md`, 2.4). Agents are
hosted in that loop and joined to their worker by real pipes. The store
and client credentials use a scratch directory on the real filesystem.

The first stories sign in, start a chat and wait for its reply, and
satisfy one client's wait by another client's act; each drains with
everything settled. Then a service restart retains the work, and a
call whose answer was lost, retried under its named key, is made once.
The named-key story waits for jig's contract (section 13). The fake
forge's API joins stories that need it; clone, push and landing stories
wait for its real-git bridge (`testing.md`, 2.4 and section 10).

These are a small set of focused tests at the gate. They reuse lower
worlds' scripts and outside expectations where applicable, and add
checks of the full transport, persistence and teardown on the real
kernel. Most tool behaviour stays in `tests/client`; the harness checks
memory ownership, outstanding operations and release between iterations.
A real-loop failure is kept with its trace and reproduced lower down
where possible; its seed does not replay the kernel's schedule.

### 11.4 The process tier

`temper serve` and `temper ctl` as processes over skein's fake LLM peer
(`testing.md`, 2.6), on a scratch directory of their own: sign-in by the
printed link; a repository adopted; a chat asked for a change and waited
for to its end; a wait answered by another process's act; an act cut by
a termination signal, then asked again under its key, made once; the
exit statuses; two acts at once on one DIRECTORY, serialized by the
lock.

The executable boundary is checked here: argv through `main`, printed
output, exit statuses, the directory's lock, OS signals and agent
process launch. The owned Forgejo and runner prove the change against
real git and CI (`testing.md`, 2.6).

### 11.5 The agents' guide

`temper ctl help` prints a short guide, then the tools from the table:
how to sign in, the loop of 5.4, naming keys and their prefix, reading
acts, the exit statuses. The guide is text of `temper-client-tools`; a
test fails when it names a tool the table lacks.

### 11.6 Findings into stories

Each bug an agent finds through the tools ends as a story in
the lowest tier that shows it, before its fix lands: a domain world,
`tests/client`, the simulated world, the real loop or the process tier
(`draft/qa.md`, section 1). A story's calls are the tools' calls, so an
agent's history of `temper ctl` commands transcribes into one.

## 13. What it asks of jig

Area 1 needs two things of jig and skein, which the tools do not work
around:

- **A named key.** `jig-client`'s `Send` under an engine key the
  application gives, in place of the one the request table draws (jig's
  `client/domain/client.md`, section 6), which needs skein's request
  table to admit a request under a given key, refused when it holds that
  key already (skein's `lib.md`, section 12). Everything else of keyed
  requests stays: kept before sent, sent again until durable, answered
  once, a second ask under the key a conflict (jig's
  `client/domain/contract.md`, 3.1).
- **A read of one turn.** A contract read, `turn{n, attempt, turn}`
  (jig's `client/domain/contract.md`, section 6), answered from the
  store's `Scope::Turn` (jig's `protocol/store.md`, 7.3) with the turn
  decoded, within the client's limit for one document, and with its size
  in its place past it.

Everything else the tools use is jig's as designed: its helpers
(`shapes`, `facts`, `thread`, `words`), the documents' acts and `me`'s
bounds (jig's `client/domain/contract.md`, 5.3), the client's bound on a
path's segments, a service's sign-in, and the one-shot mode, whose modes
"differ in the host alone" (jig's `client/domain/client.md`, section
20).

## 14. Later

Each is named here and designed when it comes.

- **An MCP endpoint of the engine,** for agents with no shell:
  `temper-service` serves the same tools at `/mcp` on the clients'
  listener, each session a tools' root holding `jig-client` over an
  in-process transport to jig's clients' component (jig's
  `protocol/service.md`, section 10), signed in by its bearer, a wait a
  long call. It needs jig's client in the engine's process, and an
  agent other than a shell agent to drive temper. Area 1 leaves it
  wiring: a call decoded from JSON (`--call`), the rendering told how to
  print a call, a call per asker.
- **Extraction into jig,** `temper-client-tools`' module `jig` moved to
  `jig-client-tools`, when a second application wants the tools.
- **The forge's read family's tools,** as the engine serves each of the
  forge's parts (`web/ux/forge.md`, section 9).
- **A service's credential reissued, or a service removed** (jig's
  `domain/people.md`, section 3): tools of area 3.

## 15. Open questions

- **`temper open` and `temper ctl` on one DIRECTORY:** not supported and
  not detected (`client.md`, section 11). Default: the lock of 8.3 is
  `temper ctl`'s alone, until `temper open` takes it.
- **The one-shot host's place:** temper's, in `temper-client-shell-cli`.
  Default: it moves into `jig-client-shell-native` with the extraction
  (section 14), if jig wants one.
