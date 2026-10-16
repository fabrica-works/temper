# Tools: temper for agents

Draft, 2026-10-16. An interface through which agents drive temper:
reading what it holds, acting through jig's operations, and waiting for
what changes. It is first a command line, `temper ctl`, for agents that
QA and develop temper's core; later the same tools are served by an
MCP endpoint of temper's engine, for agents that drive a production
temper, on a person's behalf or as a party of their own. It assumes
temper as the client-api plan leaves it (temper `main` at 02437a99b,
that plan running): temper's client on `jig-client`, temper's server
owning the clients' listener. Nothing here is adopted: `client.md`,
`protocol.md` and jig's documents describe what is designed. Section 15
says what this changes in them; section 16 records the questions the
user settled.

## 1. In one page

- **Agents drive the plumbing, not the pages.** The tools are jig's
  operations, subjects and reads (jig's `client/domain/contract.md`),
  jig's sugar where it fits (`shapes`), and temper's forge family. An
  agent never reads a page, a number on a screen or a layout (section
  3).
- **Each document says what can be done next.** jig's documents carry
  the acts the party may make (made, proposable, refused); every result
  prints them as the exact calls that make them, with the revisions
  they name (section 6).
- **One table of tools, many faces.** Each tool is defined once: its
  name, its arguments, its result, its words. The command line's parser
  and help, the JSON, and later MCP's tool list and schemas are all
  derived from the table (5.1).
- **Waiting is a tool.** `wait` blocks on a watch until a subject
  changes from what the agent saw, or meets a condition, or a timeout
  passes. An agent's loop is `wait me --until waiting`, `inbox`, act
  (5.4).
- **Each request once.** Every tool that makes something requires a
  key the agent names; asked again under that name, it gets the first
  answer, never a second chat or goal. Acts that decide name the
  revision the agent saw (section 7).
- **The command line is thin.** `temper ctl DIRECTORY TOOL ...`
  (`temper-client-shell-cli`) runs one call per process on
  `jig-client`'s one-shot mode (jig's `client/domain/client.md`,
  section 20), over jig's native transport, and prints its result
  (section 8).
- **MCP later, an endpoint of the engine.** The same tools, served by
  `temper-service` on the clients' listener, each session a client in
  the engine's process (jig's `protocol/service.md`, section 10). MCP
  never runs through the command line (section 9).
- **Adding MCP is wiring.** Every tool's logic is below the command
  line from the first stage: the table, the tools' root, the rendering.
  MCP then adds an endpoint, its sessions and a transport, and no
  second implementation of any tool. Checks keep the command line thin
  (9.1).
- **Generic apart from temper's.** What is jig's (the table's jig part,
  the runner, the rendering, the MCP mapping) is a crate that names no
  temper crate, `temper-client-tools-jig`, to be renamed into jig when
  a second application wants it; temper's forge tools and words are
  `temper-client-tools` (section 4).
- **Any credential.** An agent acts with the person's own credential or
  with one of its own, a service's; the tools are the same either way
  (section 10).
- **Not here:** the person's pages, which the fake person's stories
  prove; the browser's QA, by agents driving the browser once temper
  has one; and temper's own forge, which deserves a design of its own
  (sections 2 and 11).

## 2. What it is for, and what not

- **QA and development of temper's core.** Agents use temper to do
  coding work of growing size, through growing parts of its surface:
  a chat that changes one file; a goal with a plan; proposals decided,
  questions answered, tasks held and released; adoption of a
  repository; budgets, roles and policy. What breaks is fixed, and what
  is slow is made faster, each fix with a story in the worlds
  (section 12).
- **Production, later:** an agent driving temper for a person (asking,
  following, deciding what the person delegated); an agent using
  temper as its tool for a large change or a new system, setting goals
  and following their plans; a service of a project acting as a party.
  The same tools serve each; the command line and MCP differ only in
  their host.
- **Not the person's pages.** temper's pages are the person's
  presentation, and the fake person's stories in `tests/client` prove
  them (`client.md`, section 10). The tools use none of them, so they
  wait for no page: the board, the changes and an agent's task page,
  which the client-api plan leaves to `client-pages`, are no tool's
  prerequisite.
- **Not the browser.** Once temper has a web UI, its QA is a separate
  effort, by agents driving the browser itself, as a person uses it,
  on skein's browser harness or a browser driver. These tools are not
  a way around the browser.
- **Not the deployment under test.** Starting temper with a forge, a
  runner and a model to drive is temper's own forge (section 11).

## 3. Why the plumbing, not the pages

- **The pages are for people.** The text view's numbers are positions
  that change between prints, the banner and the navigation cost an
  agent's context on every read, and a page shows only what its
  mockup chose. `draft/qa.md`, 4.2 proposed the text view as the
  driver; this draft replaces it for the core (section 15).
- **jig's client was made for this.** It is presentation-agnostic: "a
  browser, a terminal, a chat bot, a one-shot command and a service
  with no presentation hold the same client" (jig's
  `client/domain/client.md`, section 1). Its documents are typed data
  with stable numbers, revisions and offered acts (jig's
  `client/domain/contract.md`, 5.1 to 5.3).
- **Offered acts replace discovery.** A person learns what they can do
  from buttons; an agent learns it from the document: each act made,
  proposable (and to whom), or refused (and why), with the bounds a
  draft is compared with. The tools print them, so an agent needs no
  map of the interface beyond the tools' list.
- **The engine judges again.** Whatever an agent sends, the engine
  checks it as it checks a person's request: its role, its pool, the
  revision it names. The tools add no authority and remove no check.

## 4. The crates

```
temper-client-tools-jig      generic: jig's part of the table, the call runner over jig-client,
│                            marks and conditions, the rendering (text, JSON), the argv parser
│                            and help, the MCP mapping; names no temper crate
├── jig-client               jig's client, its helpers (shapes, facts, thread, words)
└── skein-json               the JSON writer and tokenizer
temper-client-tools          temper's: the forge's tools, temper's words, the table composed,
│                            the tools' root, their limits
├── temper-client-tools-jig
└── temper-client-protocol-wire  the forge family's records (client.md, 4.6)
temper-client-shell-cli      the command line: argv to a call, the one-shot host over
│                            jig-client-shell-native, the directory's lock, printing, exit status
├── temper-client-tools
└── jig-client-shell-native  jig's native transport: HTTP, the small store's file, the token's
temper                       the binary: `temper ctl` runs the command line
temper-service               later: the MCP endpoint on the clients' listener (section 9)
```

- **Named by the client naming rule** (jig's `README.md`, section 15):
  each is a client crate, holding `jig-client`; the tools are to agents
  what `temper-client-view` is to people, under the kind `tools`. The
  MCP endpoint runs in the engine's process and lives in
  `temper-service`, which keeps the engine's name.
- **The generic crate names no temper crate.** A boundary test fails
  the gate when `temper-client-tools-jig` depends on a crate named
  `temper-*`, or its source names temper. Extraction is then a rename,
  `temper-client-tools-jig` to `jig-client-tools`, as skein-ui moved
  from jig to skein; jig adopts it when a second application uses it
  (`ops`, the factory's lab), by its rule for sugar (jig's `README.md`,
  section 10).
- **The tools' root** (`temper-client-tools`) is a client root, a step
  machine with jig's one shape (jig's `client/domain/root.md`, section
  3): `jig-client` held; two children, each an owner and an asker: the
  runner, `temper-client-tools-jig`'s, and the forge's tools; routing
  by asker. It holds no page and no view, as a service's root holds
  none (jig's `client/domain/root.md`, section 11).
- **temper's words** for jig's enums are temper's phrasebook, which the
  view and the tools then share (`client.md`, 5.1).
- **Step code under the subset:** the runner, the rendering, the
  table, the tools' root. **Ordinary Rust:** the command line's host
  and the MCP endpoint's.

## 5. The tools

### 5.1 One table

- **A tool is a record of the table:** its name; a line saying what it
  does; its arguments, each a name, a type (a number, a word from a
  set, a text, a file, a duration), whether it is required, and a line;
  its result, a typed record; whether it is keyed (section 7); and its
  stage (5.5).
- **Everything else is derived from it:** the command line's parser
  and `temper ctl help [TOOL]`; the JSON encoding of arguments and
  results; MCP's `tools/list`, with each tool's input and output
  schemas; the instructions agents load (12.4). A test checks that
  each derivation names every tool.
- **A call is a value:** a tool's name and its arguments, decoded and
  checked against the table before anything is sent. The runner takes
  a call and gives back its result; neither host interprets either.

### 5.2 Reads

A read answers with a document, or a page with a cursor, and its mark
(5.4). It needs no key.

| Tool | What it answers | jig's |
|---|---|---|
| `me` | the party, its projects with their roles and pools, what waits for it counted, the templates, the bounds of what it may create, the retention | `me` |
| `inbox [--project P] [--kind K] [--root N] [--after C]` | what needs the party, then updates, each with its acts | `inbox` |
| `tasks --project P [--label chat\|goal] [--tracked] [--mine] [--ended] [--order activity\|priority] [--after C]` | task lines | `tasks{query}` (contract.md, 6.1) |
| `task N [--tree] [--activity K] [--run]` | a task's document, with its acts; optionally its live tree, its K latest activity lines and its live run, in one answer | `task{n}`, `tree{root}`, `activity{n}`, `run{n}` |
| `chat N [--before C]` | a conversation: turns, words and provisional text in order | `activity{n}`, `run{n}`, through `thread` |
| `activity N [--before C] [--limit K]` | a task's activity, a page from its end | `activity{n}` |
| `run N` | a task's live run: its attempt, its provisional text, its current call | `run{n}` |
| `proposal P/K` | a proposal, by its proposer and number, with its acts and what deciding it does | `proposal{key}`, contract.md 5.4 |
| `project P` | its policy (templates' overrides among it), roles, spend, notes | a project's reads |
| `deployment` | hosts, accounts, capacity | `deployment` |

- **Projects by name or number;** a name is resolved through `me`.
- **The forge's changes are read on the forge,** through its API or
  `git`, as a person reads them there: temper's forge family serves
  only adoption (`client.md`, section 4).

### 5.3 Acts

Each act is one keyed request through `jig-client`'s request table,
answered once durable (section 7). `K` marks the tools whose key the
agent must name.

| Tool | What it does | jig's | K |
|---|---|---|---|
| `chat --project P TEXT` | starts a chat | `shapes::chat`, a `Create` | yes |
| `message N TEXT` | words to a task | `Message` | yes |
| `close N [--move D]...` | closes a chat: its live delegates cancelled, or moved to the party where named | `shapes::close_chat`, a compound request | yes |
| `goal --project P [--priority I] [--budget A \| --as-allowed] [--deadline T] TEXT` | sets a goal, or proposes it where the party's authority does not cover it | `shapes::set_goal`, a `Create` or a `Propose` | yes |
| `decide P/K accept\|reject\|pass-up [REASON]` | decides a proposal | `Decide` | |
| `decide N --hold R release\|leave\|pass-up [REASON]` | decides an escalation | `Decide` | |
| `answer N M TEXT` | answers a question, by its task and message | `Answer` | |
| `release N --hold R [--again]` | lifts a hold | `Release` | |
| `stop N` | cancels a task's run, the task held | `Stop` | |
| `amend N --spec R CHANGE... REASON` | amends a task's spec, authority or priority | `Amend` | |
| `cancel N [--grace D] REASON` | cancels a task and its tree | `Cancel` | |
| `move N REASON` | makes the party a task's requester | `Move` | |
| `seen ...` | moves a read position | `Seen` | |
| `withdraw P/K` | withdraws the party's own proposal | `Withdraw` | |
| `take N`, `hand-back N`; `answer N --rev R ...` | a person task: taken, handed back, answered | `Take`, `HandBack`, `Answer` | |
| `adopt --project P --forge F --owner O --name N [--role owned\|fork\|context] [--home]` | adopts a repository | the forge family's request (`client.md`, 4.4) | yes |
| `create --project P FILE`, `propose --project P FILE`, `compound --project P FILE` | jig's plumbing whole: a batch, a proposal, a compound request, as JSON | `Create`, `Propose`, `Compound` | yes |
| the project's own: `policy`, `role`, `pool`, `service` | as jig's `domain/people.md`, 5.2 | the project's operations | yes |

- **The arguments are jig's,** named as its operations name them, with
  the revisions they need (jig's `client/domain/contract.md`, 5.2): a
  proposal is decided by its number; an escalation and a release by
  the hold's revision; an amendment by the spec's; a person task's
  answer by its revision. An agent copies them from the acts a result
  printed (6.1); it never has to know which an act needs.
- **A draft beyond the party's bounds** (creating, amending) is
  compared with the bounds the document gives, as the person's client
  does (jig's `client/domain/contract.md`, 5.3): within, sent; between
  the bounds, sent as `Propose` where it is proposable, and the result
  says so; past them, refused by the client, nothing sent.
- **No confirmation.** The person's client says what an act will do
  before sending it (`client.md`, 3.6); an agent reads it in the
  document's acts and their effects, and the revision is its promise.

### 5.4 Waiting

```
wait SUBJECT [--since MARK] [--until CONDITION] [--timeout DURATION]
```

- **The subject** is a watch's (jig's `client/domain/contract.md`,
  7.1): `me`, `inbox`, `task N`, `tree N`, `run N`, or a task query.
- **A mark** is a digest of the subject's document as the engine
  delivered it, printed with every read of that subject and every
  wait's answer. jig's revisions move only with what a party decides
  about (contract.md, 5.2), so they cannot say whether a document
  changed; the mark can, and needs nothing of jig.
- **`--since MARK`:** answered at the first delivery whose mark
  differs; without it, at the first change after the watch's first
  delivery.
- **`--until CONDITION`:** answered at the first delivery that meets
  it, the first included. Conditions are facts over the document, few
  and named:

  | Subject | Conditions |
  |---|---|
  | `me` | `waiting` (something waits for the party) |
  | `inbox` | `waiting`; `updates` (an unread update) |
  | `task N` | `ended`; `waiting-on-me`; `held`; `phase:<phase>` |
  | `tree N` | `waiting-on-me` (a proposal or a question below waits for the party); `ended` |
  | `run N` | `ended` |

- **`--timeout`,** 300 seconds by default and at most 3600: answered
  with the current document, its mark, and that the condition is not
  met (6.3).
- **Level-triggered, as the watch is** (contract.md, 7.2): deliveries
  between are coalesced, and the answer is the document as it is when
  answered. Nothing is missed: a change before the wait began is in
  its first delivery, which `--since` compares.
- **The agent's loop:** `wait me --until waiting`; `inbox`; act; again.
  An agent following one goal: `wait tree N --until waiting-on-me`, or
  `wait task N --until ended`.

### 5.5 Stages

The tools come in the order the QA effort uses temper's surface:

1. **A chat:** `sign-in`, `me`, `inbox`, `tasks`, `task`, `chat`,
   `activity`, `run`, `wait`; `chat`, `message`, `close`. An agent asks
   temper's chat for a change and follows it to its end.
2. **Goals and decisions:** `goal`, `task --tree`, `proposal`;
   `decide`, `answer`, `release`, `stop`, `amend`, `cancel`, `seen`,
   `withdraw`.
3. **The forge:** `adopt`.
4. **The project's own and jig's plumbing whole:** `project`,
   `deployment`, `move`, person tasks, `create`, `propose`, `compound`,
   `policy`, `role`, `pool`, `service`.
5. **MCP:** the endpoint (section 9).

## 6. What a result looks like

### 6.1 Text

The default: plain UTF-8 lines, no colour, no wrapping. A document
prints its header line, its fields, its acts and its mark. For
example, a goal waiting on the agent:

```
task 41 · goal · running · shop · priority 2
asked: Add a /health endpoint that reports the store's state
requester: you · spent 1.20 of 5.00 · no deadline
waiting on: you: question 3 of task 44, "Which port should it listen on?"
plan: 4 live (2 running, 1 waiting, 1 held below)
acts:
  answer 44 3 TEXT
  amend 41 --spec 2 CHANGE... REASON
  stop 41
  cancel 41 REASON
mark: task 7c1e09a2
```

- **Acts as calls,** in the host's syntax: a command line's words for
  `temper ctl` (with the prefix the agent typed left out), a tool's
  name and arguments for MCP. A proposable act says to whom; a refused
  one says why, and prints no call.
- **jig's words, temper's phrasebook:** phases, holds, what a task
  waits on and refusals in the words a person reads (`words`), so an
  agent and a person name things alike.
- **Lists** print one line per row, its number first, and the cursor
  for the next page, or that there is none.
- **An act's outcome** prints what was made or decided and its
  numbers: `done: task 41, goal, budget 5.00 given`; `proposed: 3/17,
  to maintainers`; `decided first: by ada, accepted, 14:02`.

### 6.2 JSON

`--json` prints the result record instead, one JSON document on
stdout, written with `skein-json`'s writer: the result's fields by
their names in the table, numbers as jig's integers, no floats. It is
the contract for scripts and MCP's structured content; the text is for
reading, and may change with temper's words.

### 6.3 Refusals and exit status

| Status | Meaning | Prints |
|---|---|---|
| 0 | done: a read answered, an act done or proposed, a wait's condition met | the result |
| 1 | the call failed: usage, the directory refused, the engine unreachable, signed out, the client's limit, unknown past the retention | one line on stderr, `temper ctl: ...` |
| 2 | refused by the engine | jig's fact, typed in JSON, and its words; the act's alternatives where the refusal names them (look again, propose instead) |
| 3 | a wait's timeout, its condition not met | the current document and its mark |

### 6.4 Bounded

- **Every result has a ceiling,** in rows and bytes, from the table's
  windows: a list's page, the activity lines a `task` adds, a
  conversation's page. A text cut prints how much was left and the
  call that reads the rest.
- **Long texts are paged, never whole by default:** run text, activity
  and conversations take `--before` and `--limit`. An agent's context
  is the budget the defaults are chosen for.

## 7. Requests: once, and named

- **Through jig's request table.** Every act is a keyed request:
  `jig-client` keeps it before sending it, sends it again under its
  key until its answer is durable, and answers it once (jig's
  `client/domain/client.md`, section 6).
- **The agent must name the key** of every tool that makes something
  new (`K` in 5.3): `--key NAME`, a word of the agent's, such as
  `health-goal`; a call without it is refused as a usage error, before
  anything is sent. The same name asked again, by the same party, within
  the engine's retention, is the same request: a rerun after a lost
  answer, a timeout of the agent's own tool, or a context the agent
  lost gets the first answer, never a second chat. The same name with
  another ask is refused as a key conflict.
- **A named key is the engine's key,** derived from the name and the
  tools' namespace, so a rerun is the same request in any process, the
  command line's or the engine's MCP endpoint's, with nothing kept on
  the agent's side. This asks jig for a `Send` whose engine key the
  application gives (section 14).
- **Acts that decide name the revision the agent saw;** a moved one is
  refused, and the agent reads again. A repeat of a decision is
  answered as decided first, by the party itself.
- **Tools without a named key** (`stop`, `cancel`, `decide`, ...) take
  `--key` too, optionally; without it, the runner draws one, which the
  small store keeps until answered (8.3).

## 8. The command line: `temper ctl`

### 8.1 Its shape

```
temper ctl DIRECTORY TOOL [ARGUMENT...] [--json]
temper ctl DIRECTORY --call FILE [--json]
temper ctl help [TOOL]
```

- **`--call FILE`** reads one call as JSON, `{"tool": ...,
  "arguments": {...}}`, from FILE, or from stdin for `-`: the shape of
  MCP's `tools/call`. Scripts have a way in, and the decoding MCP needs
  is used and tested from the first stage (9.1).
- **DIRECTORY** is the client's private directory, as for `temper
  open` (`client.md`, 6.1): its credential, `token`, and its small
  store, `store`, both `jig-client-shell-native`'s, `0o700` and
  refused unless the user's own.
- **`sign-in LINK`** signs in by the link `temper serve` printed, on
  loopback, and writes the token, as `temper open` does (`client.md`,
  section 7); `sign-in --service FILE` with a service's credential read
  from FILE, never from an argument (section 10). `sign-out` ends the
  sign-in and erases the token. Every other tool signs in with the
  token DIRECTORY holds (`SignIn { with: Held }`).
- **The engine's address** is the one the sign-in's link named, kept in
  DIRECTORY beside the token. On loopback only, as `temper open`'s,
  until the clients' listener serves TLS.
- **No configuration file;** its inputs are its arguments and
  DIRECTORY.

### 8.2 One call, one process

The host of `jig-client`'s one-shot mode (jig's
`client/domain/client.md`, section 20), ordinary Rust on skein's loop,
as the terminal's host is:

1. parse the arguments against the table; a usage error exits 1 before
   anything is opened;
2. open DIRECTORY; for a call that sends, take its lock (8.3) and load
   the small store's records;
3. start the tools' root, sign in with the token, wait for `me`;
4. send what the store kept, each answered as a notice line,
   `kept: ...`, before the call's result;
5. run the call: a read answered, an act's answer durable, a wait's
   condition met or its timeout;
6. print, close the stream, exit with the call's status.

- **A termination signal** stops the call; what it sent is kept in the
  small store, and the next call sends it again under its key.
- **`pending`** lists what the small store keeps, and its progress.

### 8.3 Several calls at once

An agent runs a `wait` while it acts, or several agents share a party.

- **A call that sends** takes an exclusive lock on DIRECTORY's store
  for its run, so two never write the small store over each other
  (`client.md`, section 11 names the lock as the candidate).
- **A read and a wait** take no lock and start the client with nothing
  kept: they write no record and send nothing but reads and watches.
- **Several parties** are several directories, each signed in as its
  own.

## 9. MCP, later: an endpoint of the engine

### 9.1 Wiring, not a second client

Adding MCP must cost little: an endpoint, not a second implementation
of the tools. It does when the command line is layered as section 4
has it, and these hold from the first stage, each checked:

- **The command line holds no tool.** `temper-client-shell-cli` turns
  its arguments into a call through the table, hands the call to the
  tools' root, prints the rendering, and exits with the status of the
  outcome's class (6.3). It matches on no tool and no result. A
  boundary test fails the gate when its source names a tool of the
  table.
- **A call decodes from JSON as from arguments,** through the table;
  `temper ctl --call` uses it (8.1), so MCP's `tools/call` finds it
  built and tested.
- **The table gives MCP's tool list:** its input and output schemas are
  golden text from the first stage (12.1).
- **The root runs several calls at once,** each its own asker,
  although the command line makes one: an MCP session's calls in
  flight need nothing new of it.
- **The root runs with a small store or without one:** the command
  line's host gives it one, the endpoint's none (section 7; 9.2).
- **The host chooses how an act's call is printed** (6.1): the
  rendering takes it as a parameter.
- **The same stories run through each host:** the scripted agent's
  stories run against the tools' root in `tests/client`, through
  `temper ctl` in the process tier, and, once it exists, through
  `/mcp`, with the same calls and the same results (section 12).

What MCP then adds is the endpoint alone: JSON-RPC over HTTP, a table
of sessions, the bearer given to each session's client as held, and the
in-process transport's arms. The last is the one piece that is not
temper's wiring: jig's client in the engine's process, which jig's
`protocol/service.md`, section 10 designs as a few arms of the routing,
and which jig's client-api `later.md` defers to the first application
that runs one (section 14).

### 9.2 The endpoint

- **Served by `temper-service`** on the clients' listener its server
  owns (`client.md`, section 8), beside jig's `/jig`, at `/mcp`: MCP's
  HTTP transport, JSON-RPC 2.0 over `POST`, with server-sent events for
  a call that reports progress.
- **Each session holds a tools' root,** the same as `temper ctl`'s,
  holding `jig-client` over the in-process transport to jig's clients'
  component (jig's `protocol/service.md`, section 10): exchanges and
  the stream as messages' bytes, no HTTP inside the process. The agent
  never reaches the engine through the command line, and the command
  line never through MCP.
- **Signed in by its bearer:** `Authorization: Bearer`, a credential
  the clients' component minted (the printed link's sign-in, a
  service's), given to the session's client as held. jig checks every
  exchange's credential, whatever carried it.
- **The table is the tool list.** `tools/list` gives each tool its
  name, its line, and its input and output schemas from the table;
  `tools/call` is a call, answered with the text of 6.1 as its content
  and the record of 6.2 as its structured content. An act's call in a
  result is printed as a tool's name and arguments.
- **A wait is a long call,** answered when its condition is met or its
  timeout passes, with progress notifications while it waits. Watches
  are not MCP's resources and subscriptions, which clients support
  unevenly (section 16).
- **No small store.** A session's requests with a named key are safe to
  ask again after a lost session (section 7); one without is lost with
  its session, answered unknown if asked about.
- **Bounded:** sessions per party and in all, calls in flight per
  session, each call's result, all within the engine's limits; past
  them, refused as busy.
- **Guards:** the clients' listener's, as jig's example guards are, and
  MCP's `Origin` check, so a browser page cannot drive a local
  temper. On loopback until the listener serves TLS.
- **Where the code is:** the MCP mapping (the tool list, a call
  decoded from JSON, a result encoded) is pure, in
  `temper-client-tools-jig`; the endpoint (HTTP, sessions, the
  in-process routing) is `temper-service`'s.

## 10. Who the agent is

- **Any credential.** An agent working "on a person's behalf" may hold
  the person's own credential or one of its own: the tools, their
  calls and their results are the same either way, and jig judges each
  request as the party whose credential it is (jig's
  `domain/people.md`, section 3).
- **The person's own:** the printed link's sign-in, in standalone. The
  QA effort's first identity is the deployment's first owner.
- **Its own, a service's:** an owner makes a service in a project, with
  a role, and jig mints its credential once (`service --project P
  --role R`, written to a file the owner names, never printed). The
  agent signs in with it (`sign-in --service FILE`) and acts within
  that role's authority and pool. QA plays several parties this way.
- **Credentials are files,** private, never arguments, never printed,
  as the terminal's token is (`client.md`, section 7).

## 11. Beside it: temper's own forge

The deployment the agents drive (a forge, its CI runner, temper, a
model) is a product feature of its own, not a test harness: temper
offers to start, own and configure Forgejo and its runner for a
deployment, as it owns its store. It deserves its own design and plan,
after this draft. What the tools need of it:

- a deployment an agent can start, sign in to and throw away, with a
  repository to adopt;
- the forge readable by the agent directly (its API, `git`), since
  temper serves no forge part but adoption;
- the engine's and the worker's logs where the agent can read them,
  outside the tools.

Until then, QA runs against a deployment set up by hand
(`draft/mvp.md`'s milestone), and resetting it is the harness's, never
a tool's.

## 12. Testing

### 12.1 Step tests

- **The table:** every tool's argv parsed into its call and back; usage
  errors; help; the JSON schemas and MCP's tool list, as golden text.
- **The rendering:** each result kind in text and JSON, as golden text,
  with temper's words.
- **Marks and conditions:** a mark moves with a document's change and
  only then; each condition over the documents that meet and miss it.

### 12.2 `tests/client`

`temper-client-world` runs the tools' root against temper's engine
root, beside the fake person, as a scripted agent making calls
(`client.md`, section 10): from stage 1, a chat started, its reply
waited for, a message, the chat closed; then each stage's stories. The
world's faults hold: a call cut by a link drop or an engine restart,
asked again under its named key, is made once.

### 12.3 The process tier

`temper serve` and `temper ctl` as processes over skein's fake LLM
(`testing.md`, 2.6): each stage's story, the exit statuses, a wait
answered by another process's act, the directory's lock.

### 12.4 The agents' instructions

`temper ctl help` prints the whole vocabulary, generated from the
table. A short guide for agents (how to sign in, the loop of 5.4,
naming keys, reading acts) ships with temper; a test fails when it
names a tool the table lacks, or the table a tool it lacks.

### 12.5 Findings into stories

Each bug an agent finds through the tools ends as a story in
`tests/client`, the process tier or a domain world, before its fix
lands (`draft/qa.md`, section 1). A story's calls are the tools'
calls, so an agent's history of `temper ctl` commands transcribes into
one.

## 13. Order of building

1. **After client-api, stage 1** (5.5): the three crates, `temper ctl`,
   the named key (section 14), the stories of 12.2 and 12.3 for a chat.
   The QA effort starts here, with small coding tasks through a chat.
2. **Stages 2 to 4,** each when the QA effort reaches it, each with its
   stories.
3. **MCP,** once jig's client in the engine's process is built (jig's
   client-api `later.md`, the row "A client in the engine's process"),
   and an agent other than a shell agent drives temper: the endpoint
   and the stories through it, wiring only (9.1).
4. **Extraction into jig,** when a second application wants the tools.

## 14. What it asks of jig

- **A named key:** `jig-client`'s `Send` with an engine key the
  application gives, derived from a name, in place of the one the
  request table draws (jig's `client/domain/client.md`, section 6;
  skein's `lib.md`, section 12). Everything else of keyed requests
  stays: kept before sent, sent again until durable, answered once.
- **Nothing for the one-shot mode:** the modes "differ in the host
  alone" (jig's `client/domain/client.md`, section 20).
- **For MCP:** the client in the engine's process (jig's
  `protocol/service.md`, section 10), which jig's client-api `later.md`
  defers to the first application that runs one.
- **For an agent's own identity:** services, as jig's
  `domain/people.md`, section 3 designs them, where not yet built.
- **Later:** `jig-client-tools`, renamed from
  `temper-client-tools-jig`.

## 15. What it changes, if adopted

- **`protocol.md`, section 4:** a fifth command, `temper ctl DIRECTORY
  TOOL ...`, in the process tree beside `temper open`; section 5: the
  three crates; later, the MCP endpoint in `temper-service`.
- **`client.md`:** section 2, the tools' crates beside the view's;
  section 11, "Clients that are not a person's", answered by the tools
  and services; the lock on DIRECTORY of section 11, taken by `temper
  ctl`.
- **`testing.md`:** the scripted agent in `tests/client` and the
  process tier's `temper ctl` stories.
- **`draft/qa.md`, 4.2:** the driver of temper's core is these tools;
  the pages are the fake person's, and the browser's QA drives the
  browser. `draft/factory.md`, section 13, likewise.

## 16. Settled

The user settled the draft's questions on 2026-10-16:

- **The named key is asked of jig** (section 14). The alternative, a
  record of each name, its ask's digest and its answer kept by the
  command line beside the small store, would need a second record for
  MCP's sessions.
- **Keys are required** on the tools that make something new: a call
  without `--key` is refused before anything is sent (section 7).
- **The command is `temper ctl`,** with DIRECTORY as its first
  argument, as `temper open` has it: no environment variable, no
  default directory.
- **Any credential:** the person's own or the agent's, a service's
  (section 10).
- **Several people in QA are services,** each a party with a role;
  people signing in through the forge come with temper's `Mint`
  (`client.md`, section 11).
- **Loopback only,** for the command line and MCP, until the clients'
  listener serves TLS, which waits for skein's TLS server side (jig's
  client-api `later.md`).
- **Waiting is the `wait` tool,** for MCP too; not MCP's resources and
  subscriptions, which clients support unevenly.
- **The JSON is the contract;** the text follows temper's words, and
  may change with them.
