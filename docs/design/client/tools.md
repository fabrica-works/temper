# Tools: temper for agents

Provisional, 2026-10-17, revised 2026-10-19. How agents drive temper:
they read what it holds, act through jig's operations and wait for what
changes, with `temper ctl`, a command line. It serves the agents
(smith's) that QA and develop temper's core: they drive a throwaway
deployment on coding tasks of growing size, then on temper's features,
until the first milestone (`../README.md`, section 2) is usable for real
tasks. Their QA loop extends the tools area by area (5.5). Their bounds
are section 12, and what they take of jig section 13; an MCP endpoint
and the tools' extraction into jig are later (section 14). What is
still open is listed in section 15.

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
  (section 7). The named key is jig's (section 13).
- **A turn too large for a page is read alone,** by jig's read of one
  turn (5.2, section 13).
- **Thin command line.** `temper ctl DIRECTORY TOOL ...` makes one call
  per process, in `jig-client`'s one-shot mode, over jig's native
  transport, and prints its result (section 8).
- **The deployment under test is a box** (`../protocol/setup.md`, section
  7): the agent's harness starts it, signs in by its printed link, adds
  the deployment's LLM key, and throws it away (section 10).
- **LLM accounts are an area of their own** (5.5, area 5): a sign-in
  prints the address to open and returns, and the agent follows it with
  `wait accounts`; a key is read from a private file and sent in a
  request that is never kept (section 7).
- **Grown by areas,** the first and the fifth designed in full, the
  others named (5.5); **later, named:** the MCP endpoint and the
  extraction into `jig-client-tools` (section 14).

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
  the fake person's stories in `tests/client` (`README.md`, section 10).
  No tool waits for a page: a tool reads jig's documents, so no page is
  a tool's prerequisite.
- **Not the browser.** Once temper has one, its QA is agents driving the
  browser as a person does. These tools are no way around it.
- **Not the deployment.** Starting temper with a forge, a runner and a
  model is `../protocol/setup.md`'s; resetting it is the harness's, never a
  tool's (section 10).

## 3. Why the plumbing, not the pages

- **The pages are for people.** The text view's numbers are positions
  that change between prints, the banner and navigation cost an agent's
  context on every read, and a page shows only what its mockup chose. So
  the text view is no agent's driver; these tools are, for temper
  (`../draft/qa.md`, 4.2).
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
temper-client-tools          the table, a call, the runner, digests and conditions, the rendering (text,
│                            JSON), the argv parser and help, the guide, the limits; the tools' root
├── jig-client               jig's client and its helpers (shapes, facts, thread, words)
├── temper-client-domain     the forge's client side and its words (README.md, 3.1, section 4, 4.1 and 4.7)
├── temper-client-protocol-wire  the forge family's records (README.md, 4.6)
├── skein-ui                 phrases: jig's words and the tools' own, rendered as text
└── skein-json               the JSON writer and tokenizer
temper-client-shell-cli      the host: argv to a call, the one-shot host over jig-client-shell-native,
│                            the directory's lock and address, printing, exit status
├── temper-client-tools
└── jig-client-shell-native  jig's native transport: HTTP, the small store's file, the token's
temper                       the binary: `temper ctl` runs the host
```

- **Named by the client naming rule** (jig's `README.md`, section 15):
  the tools are to agents what `temper-client-view` is to people.
- **jig's part apart from temper's.** `temper-client-tools`' module
  `jig` holds what is generic: jig's part of the table, the call, the
  runner, digests and conditions, the rendering. It names no temper item:
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
  none (jig's `client/domain/root.md`, section 11). The runner is one
  owner, `Runner`, 3 (`README.md`, 4.6), and so one asker (jig's
  `client/domain/root.md`, section 15). Each handle and key it gives
  the client carries its call's number in the high 32 bits of its
  number, calls numbered from one past the highest among the keys
  kept at `Start`: several calls in flight need nothing new of the
  root, and a new key never meets a kept one.
- **Words:** jig's for jig's enums (`jig-client`'s `words`) and the
  forge's of `README.md`, 4.1 and 4.7, so an agent and a person name
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

A read answers with a document, or a page and its cursor, and, for a
watch's subject, its digest (5.4). It needs no key.

| Tool | What it answers | jig's |
|---|---|---|
| `me` | the party, its projects with their roles and pools, what waits for it counted, the bounds of what it may create, the retention | `me` |
| `inbox [--project P] [--kind K] [--root N] [--after C]` | what needs the party, then updates, each with its acts | `inbox` |
| `tasks --project P [--label chat\|goal] [--tracked] [--mine] [--ended] [--order activity\|priority] [--after C]` | task lines of one group: the live tasks, or with `--ended` the ended | `tasks{query}` (jig's `client/domain/contract.md`, 6.1) |
| `task N [--tree] [--activity K] [--run]` | a task's document, with its acts; with its live tree, its K latest activity lines, its live run, in one answer | `task{n}`, `tree{root}`, `activity{n}`, `run{n}` |
| `chat N [--before C]` | a conversation: turns, words and, where the run's host gives it, provisional text, in order, a page from its end | `activity{n}`, `run{n}`, through `thread` |
| `activity N [--before C] [--limit K]` | a task's activity, a page from its end | `activity{n}` |
| `turn N A.T [--part K] [--from B]` | one turn of task N, attempt A, turn T, whole: its text, each call and its result | `turn{n, attempt, turn}` (jig's `client/domain/contract.md`, section 6) |
| `run N` | a task's live run: its attempt, its provisional text where its host gives it, its current call | `run{n}` |

- **`tasks` reads one group a call.** jig answers a task query in two
  groups, live then ended, each with its own cursor (jig's
  `client/domain/contract.md`, 6.1), and a printed cursor is one
  group's. Without `--ended`, `tasks` reads the live group, `--after C`
  its cursor; with `--ended`, the ended group alone, as jig's view reads
  it, `--after C` the ended group's. The call that reads on keeps
  `--ended`, whose help reads "Only ended tasks". An agent that wants
  both groups makes two calls; the printed cursor is jig's, with nothing
  added.
- **`tasks`' order:** by number, ascending, without `--order`, with
  `--ended` or without; by activity or priority with it. The tools never
  send jig's other orders, `Changed` and `Ended`: no command line makes
  a call with either, and such a call is refused a command line, never
  given a guessed one, so that a call's command line parses back to
  that call (6.1).
- **`inbox` reads needs, then updates,** in the one call. jig reads the
  inbox a group a call, needs or updates, each with its own cursor, and
  each kind in one group. A call reads needs first; while needs has
  more, its page is needs alone. Once needs ends on a page, updates'
  first page follows it when both fit in a page's rows (section 12);
  when they do not, the call that reads on starts updates. No page
  passes the rows, and no update is read twice.
- **The inbox's printed cursor** is one group byte, 0 for needs and 1
  for updates, then jig's cursor, whole, or none when the call that
  reads on starts updates; to the agent it stays opaque. The call that
  reads on stays in needs while needs has more, then moves to updates.
  `--after C` reads C's group; with `--kind K`, a C of the other group
  is a usage error, exit 1.
- **A page cut by `--limit K`** reads on with the same read without
  `--limit`, `activity N`, or `activity N --before C` with the C it was
  given, which prints the page whole, its K rows again, then reads on
  at jig's `before`. `task N --activity K` that cuts a page prints
  `read with: activity N`. The printed cursor stays jig's: at most a
  page's rows repeat, and no row is skipped.
- **Provisional text is the run's host's.** An agent process gives none
  (`../protocol/llm.md`, section 7), and `chat` and `run` then print none:
  a run shows by its activity, and its words as each turn commits.
- **A turn too large for a page** comes in a page by its size in place
  of its bytes (jig's `client/domain/contract.md`, 5.1).
  `activity`, `chat` and `task --activity` print it as one line, its
  number, its size and the call that reads it: `turn 3.7 · 412 KB ·
  read with: turn 41 3.7`.
- **`turn`** prints each part of the turn cut to the tools' window, each
  cut with the call that reads on (`--part K --from B`). A turn past the
  engine's bound on one read, or the client's limit for one document, is
  answered with its size and that it is past the limit, exit 0; the
  agent reports it as a finding (section 12).
- **The forge's changes are read on the forge,** with its API or `git`,
  as a person reads them there. A part of the forge's that the engine
  does not serve prints "Shown on the forge.", with the resource in the
  forge's words (`README.md`, 4.1 and 4.2). The forge's read family
  (`forge.md`, section 9) joins the table as tools of its own
  (section 14).

### 5.3 Acts

Each act is one keyed request through `jig-client`'s request table,
answered once durable (section 7). `K` marks the tools whose key the
agent must name.

| Tool | What it does | jig's | K |
|---|---|---|---|
| `chat --project P TEXT` | starts a chat | `shapes::chat`, a `Create` | yes |
| `message N TEXT` | words to a task | `Message` | yes |
| `close N [--move D]... [--reason TEXT]` | closes a chat: its live delegates cancelled, or moved to the party where named; the reason empty unless given, as jig's reference UI sends it | `shapes::close_chat`, a compound request | yes |
| `adopt --project P [--forge F] --owner O --name N --landing B [--role owned\|fork\|context] [--prefix X] [--no-home] [--no-ci] [--check G]...` | adopts a repository | the forge family's request (`README.md`, 4.4) | yes |
| `answer N M TEXT` | answers question M of task N | `Answer` | |
| `decide P/K accept\|reject\|pass-up [REASON]` | decides a proposal: P is `tN` for task N's, `pN` for party N's | `Decide` | |
| `decide N --hold R release\|leave\|pass-up [REASON]` | decides an escalation | `Decide` | |
| `release N --hold R [--again]` | lifts a hold | `Release` | |
| `stop N` | cancels a task's run, the task held | `Stop` | |
| `cancel N [--grace D] REASON` | cancels a task and its tree | `Cancel` | |

- **The arguments are jig's,** named as its operations name them, with
  the revisions they need (jig's `client/domain/contract.md`, 5.2): a
  proposal by its proposer and number, `t41/3` for task 41's third,
  `p3/17` for party 3's seventeenth, as every result prints it, an
  escalation and a release by the hold's revision. An agent copies them
  from the acts a result printed (6.1); it never has to know which an
  act needs.
- **`adopt` takes the adoption form's fields and defaults** (`README.md`,
  4.4): forge 1, role `owned`, prefix `temper/`, the home and CI on.
  `--landing` names the landing branch. A context repository is never
  the home: `--role context` turns the home off. `--no-ci` needs at
  least one `--check`, a gate number, each number once. The form's
  checks are the parser's: a call that fails one exits 1 before
  anything is sent.
- **No confirmation.** The person's client says what an act will do
  before sending it (`README.md`, 3.6); an agent reads it in the
  document's acts and their effects, and the revision is its promise.
- **A draft beyond the party's bounds** is compared with the bounds the
  document gives, as the person's client does (jig's
  `client/domain/contract.md`, 5.3): within, sent; between the bounds,
  sent as `Propose` where it is proposable, and the result says so; past
  them, refused by the client, nothing sent. Where `me` gives no bound
  for a draft, it is sent and the engine's answer printed.

### 5.4 Waiting

```
wait SUBJECT [--since DIGEST] [--until CONDITION] [--timeout DURATION]
```

- **The subject** is a watch's (jig's `client/domain/contract.md`, 7.1):
  `me`, `inbox`, `task N`, `tree N`, `run N`, `accounts` (area 5 of
  5.5), or a task query as `tasks` takes it, with what its watch takes
  alone: `inbox` with no flag, the
  inbox's watch being the party's whole inbox; `tasks` without
  `--after` or `--ended`, `tasks{query}` following live tasks. Any
  other is a usage error, exit 1.
- **A digest** is the first 8 bytes of the SHA-256 (skein's `lib.md`,
  6.2) of the subject's document as `--json` prints it (6.2), without
  its `digest` field, printed as 16 hex digits. A read and a wait digest
  one form, the watch's document: for `inbox` and `tasks`, whose read is
  a page and whose watch a window, the rows the watch's first window
  holds, in its order, and whether more follow, within the tools' page
  of rows (section 12). The read makes that form from its own rows and
  prints its cursor beside it, outside the digest. A read's digest is
  therefore a wait's first delivery's while nothing moved between them,
  so a wait `--since` a read's digest never answers at once. Two calls
  of the same tools, under the same limits, print the same digest for
  the same document. jig's revisions move only with what a party decides
  about (jig's `client/domain/contract.md`, 5.2), so they cannot say
  that a document changed; the digest can, and needs nothing of jig. It
  is not the mark the engine keeps per watch (jig's
  `client/domain/contract.md`, 7.2), which no tool prints.
- **Which reads print one:** a read whose arguments are a watch's
  subject, with every wait's answer: `me`; `inbox` and `tasks` with the
  arguments a wait takes (above); `task N`; `run N`; `accounts`.
  `task N --tree --run` prints one for each subject, `task N`, `tree N`
  and `run N`, each over its own part of the result. `--activity`'s
  lines, `chat`, `activity` and `turn` are pages, not subjects, and
  print none.
- **`--since DIGEST`:** answered at the first delivery whose digest
  differs; without it, at the first change after the watch's first
  delivery.
- **`--until CONDITION`:** answered at the first delivery that meets it,
  the first delivery included; with `--since`, at the first delivery
  whose digest differs and that meets it. Conditions are named facts
  over the document:

  | Subject | Conditions |
  |---|---|
  | `me` | `waiting` (something waits for the party) |
  | `inbox` | `waiting`; `updates` (an unread update) |
  | `task N` | `ended`; `waiting-on-me`; `held`; `replied` (a chat's run done for now); `phase:<phase>` |
  | `tree N` | `ended` |
  | `run N` | `ended` |
  | `accounts` | `usable` (an account the party's work may use has a source usable on a host); `settled` (no sign-in or check the party asked is in progress) |

- **`replied`** holds while a chat's status (jig's
  `client/domain/client.md`, section 19, its facts) is parked, waiting
  for you, waiting for them, held, closing or ended. A chat wakes on its
  person's words (jig's `domain/tasks.md`, 8.3), so a wait begun once
  `chat` or `message` has answered is not met by the reply before them:
  `message 41 TEXT --key k`, then `wait task 41 --until replied`.
- **`waiting-on-me`** covers task N's tree: what a task waits on is
  computed over its tree (jig's `domain/tasks.md`, section 3), so a
  question or a proposal below for the party meets it on `task N`, and
  `tree N` needs no condition of its own.
- **`phase:<phase>`** names the client's phase in lower case, hyphenated:
  `waiting`, `idle`, `due`, `preparing`, `claimed`, `running`,
  `backing-off`, `active`, `closing`, `held`. The ended phases, `ended`,
  `done`, `failed` and `cancelled`, are never conditions: the engine
  ends a task's watch as the task ends (jig's
  `client/domain/contract.md`, 7.2), so a delivery may never show one.
  `--until phase:done` and its kin are usage errors, exit 1, naming
  `--until ended`.
- **`settled`** follows a sign-in or a check to its end. A sign-in is in
  progress from its request's answer until it ends: made, renewed,
  refused, or past the sign-in's deadline (skein's `oauth.md`, section
  3); a check, until each host it asks has reported (jig's
  `domain/accounts.md`, 3.4). So a wait begun once `connect` or `verify`
  has answered is not met before them: `connect 1 --key chatgpt`, then
  `wait accounts --until settled`, then the account's state in its
  answer.
- **`usable`** counts the party's own accounts, those whose uses name
  the party, and the deployment's (jig's `domain/accounts.md`, 4.3), on
  any host: the box's harness waits on it once it has added the
  deployment's key (section 10).
- **`ended`** is met by a delivery that shows the subject ended, or by
  its watch's end because it ended. For `task N`, the answer is then the
  task as a read of it gives it once ended, so it says how it ended.
- **`--timeout`:** 300 seconds by default, at most 3600; answered with
  the current document, its digest, and that the condition is not met
  (6.3).
- **A watch that ends under a wait** (jig's `client/domain/contract.md`,
  7.2) answers it: met where the condition is `ended` and the subject
  ended; refused where the party may no longer see the subject; failed
  where the sign-in ended; otherwise, not met, with the last document
  and its digest (6.3). An engine that stops ends no watch: jig's
  client opens again a watch the engine let go as it stopped (jig's
  `client/domain/client.md`, 8.1), so the wait goes on while the
  engine comes back, and fails as a link down past the link's deadline
  (6.3, section 12).
- **Level-triggered, as the watch is** (jig's
  `client/domain/contract.md`, 7.2): deliveries between are coalesced,
  and the answer is the document as it is when answered. A change before
  the wait began is in its first delivery, which `--since` compares.
- **The loop:** `wait me --until waiting`; `inbox`; act; again. An agent
  following one chat: `wait task N --until waiting-on-me`, or
  `wait task N --until ended`.
- **On a box,** the forge's webhooks make the engine's documents move
  soon after the forge does; with polling alone, the same waits answer
  later (`../protocol/setup.md`, section 10).

### 5.5 Areas

The table grows in the order the QA loop reaches temper's surface. Each
area lands with its stories (section 11).

1. **A chat to a landed change** (the first QA loop, the milestone's
   story, `../README.md`, section 2): the host's tools of 5.6; every read
   of 5.2; `wait`; every act of 5.3.
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
5. **LLM accounts** (jig's `domain/accounts.md`, section 5): the party's
   accounts and, for an owner, the deployment's, as temper's accounts
   page shows them (`README.md`, 3.5). Each act is one of jig's
   requests for accounts (jig's `client/domain/contract.md`, 3.2).

   | Tool | What it does | jig's | K |
   |---|---|---|---|
   | `accounts` | the party's accounts, then, for an owner, the deployment's: each with its number, LLM provider, display name and holder; its sources in order, each with its kind, its name for a program's, and its state, per host for a program's, with the reset where the provider gave one; the runs on it and the tasks waiting for it; the sign-ins and checks in progress; its acts | the `accounts` watch (5.4) | |
   | `models [--project P]` | the models the party may use, by LLM provider: each with its efforts, what it is good at and its prices, and "needs an account" beside a provider no account of the party's work serves | `me`'s models, with the `accounts` watch's states | |
   | `connect PROVIDER` | starts a sign-in with an LLM provider: prints the address to open in a browser on the engine's machine, and the call that follows it, `wait accounts --until settled`; exits 0 once the request is durable | sign in | yes |
   | `disconnect ACCOUNT [--source S]` | signs a source out: a signed-in source's record revoked at the provider, a program's login signed out on each host; the source removed. Unconfirmed, the answer says to disconnect temper in the provider's settings, or names the hosts that did not sign out | sign out | |
   | `add-key PROVIDER FILE [--account A] [--deployment]` | adds a key read from FILE: to account A, or as a new key-only account; the deployment's with `--deployment`, by an owner | add a key | yes |
   | `replace-key ACCOUNT FILE` | replaces the account's key with the one read from FILE | replace a key | |
   | `forget ACCOUNT [--source S]` | removes a source, or the account with every source, each signed out as `disconnect` does | remove | |
   | `declare PROVIDER ADAPTER NAME [--account A]` | declares a program's login under the source's NAME (1 to 32 of `a` to `z`, `0` to `9` and `-`): a new account, or a source of account A; the answer names the command to run on each host, `temper program-login CONFIG.json NAME` (`../protocol/credentials.md`, section 3) | declare a program's login | yes |
   | `verify ACCOUNT [--source S]` | has each host check a program source's login now; answered once asked, then `wait accounts --until settled` | check | |
   | `rank ACCOUNT POSITION [--source S]` | the account's place among its holder's accounts of its LLM provider, from 1; with `--source`, the source's place in the account's order | order | |
   | `retry ACCOUNT [--source S]` | resumes a limited source, so that the runs it holds start again | resume | |

   - **The names are plain verbs, and reads nouns,** as the other
     tools' are. Six are not jig's words: `sign-in` and `sign-out` are
     the host's (5.6), so a sign-in with a provider is `connect` and its
     end `disconnect`; `forget`, `rank`, `verify` and `retry` are
     jig's remove, order, check and resume.
   - **One result per call.** `connect` prints the address and returns;
     the sign-in goes on at the engine, and `wait accounts --until
     settled` follows it (5.4). It never opens a browser, and the
     provider's redirect reaches the engine's loopback, so the browser
     that approves runs on the engine's machine
     (`../protocol/credentials.md`, section 3). `verify` alike: asked, then
     waited for.
   - **A key is read from FILE,** never an argument and never printed:
     a private file, the user's own with no group or other bits and not
     reached through a link, as the engine reads a secret's file
     (`../protocol/configuration.md`, section 6); its bytes, less a final
     newline, within the bound of section 12. The tool reads it, sends
     it once, and keeps no copy (section 7).
   - **`PROVIDER` is an LLM provider's number,** as `models` and
     `accounts` print it (jig's `protocol/configuration.md`, section 3);
     `ADAPTER` is the program's name in smith's agent configuration,
     `claude_code` (smith's `protocol/agent.md`, section 4).
   - **`S` names one of the account's sources** by its kind, `sign_in`,
     `key` or `program`, as `accounts` prints it (section 15).
   - **A program's login is never the tools'.** `declare` makes the
     source; the person, or the harness, runs `temper program-login` on
     each host, outside `temper ctl`, since it hands the terminal to the
     program's own sign-in (`../protocol/credentials.md`, section 3).
   - **The task's model** is chosen with this area: `chat` and area 2's
     `goal` take `--model M [--effort E]`, the task's LLM choice, the
     charter's default without them (jig's `domain/tasks.md`, section
     3), and area 2's `amend` changes it.
   - **An account's uses** have no tool: each takes jig's default (jig's
     `domain/accounts.md`, 4.2) until the QA loop asks for one (section
     15).

- **Areas 1 and 5 are designed here in full.** Areas 2 to 4 name their
  tools and jig's operations; each area's arguments are checked against
  jig's operations as they stand when the QA loop reaches it, by a
  design pass (skein's `development.md`, section 4), and land with it.
- **The QA loop may ask for a tool no area names.** It goes to the
  designer as a design question, and joins the table as a row of an
  area.

### 5.6 The host's tools

| Tool | What it does |
|---|---|
| `sign-in FILE\|-` | signs in by the link `temper serve` printed, read from FILE, or from stdin for `-`; writes `token` and `address` (8.1) |
| `sign-out` | ends the sign-in and leaves `token` empty, zero bytes, read as signed out (8.1) |
| `pending` | lists what DIRECTORY's store keeps: each request's operation, its party and when it was first sent, and how many records these tools cannot read |
| `help [TOOL]` | the guide, then the tools (11.5); with TOOL, its record of the table |

- **`pending` reads the store's file only:** it takes no lock, starts
  no client and sends nothing (8.3).
- **A `pending` row** is a line each: the request's operation, as a
  `kept:` line names it (6.1); its party, `party N`; and when it was
  first sent, `first sent HH:MM UTC`, the documents' words for a wall
  time (skein's `lib.md`, section 12), or `not sent`. No name and no
  key, as a `kept:` line prints none. Under `--json`: `pending`, rows
  of `operation`, `party` and `first_sent`, a wall time as jig's
  integer, or `null`; and `unread`.
- **A record these tools cannot read,** of a store version, or of a
  family's owner or version, that they do not declare, is counted, never
  listed: jig's client reads nothing of such a record, drops it at start
  and says how many (jig's `protocol/client.md`, section 10), and
  `pending` says what a call's client will. Its line, `N kept records
  these tools cannot read; a call that sends drops them`, follows the
  rows, and the call exits 0. A file that is not a small store's at all
  exits 1, "DIRECTORY's store cannot be read".
- **A TEXT of `-`** is read from stdin, to its end. A call takes at
  most one `-`, and none with `--call -`; otherwise it is a usage
  error.

## 6. What a result looks like

### 6.1 Text

The default: plain UTF-8 lines, no colour, no wrapping. A document
prints its header line, its fields, its acts and, for a watch's
subject, its digest (5.4). A goal waiting on the agent:

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
digest: task 7c1e09a2b44f0d13
```

- **Acts as calls,** without the `temper ctl DIRECTORY` prefix; a keyed
  act's call ends with `--key NAME`. A proposable act says to whom; a
  refused one says why, and prints no call. The host gives the
  rendering how a call is printed.
- **An offered act with no tool** (5.3 has none for it) prints its name
  in the tools' words and ` (no tool)`, in the document's order, with no
  call and no admission: the agent learns the act is there and may hand
  it to a person, and no call names a tool that does not exist. Under
  `--json`, the document's offers print as they are.
- **jig's words and the forge's** (section 4): phases, holds, what a
  task waits on and refusals in the words a person reads.
- **Lists** print one line per row, its number first, then the cursor
  for the next page, or that there is none.
- **An act's outcome** prints what was made or decided and its numbers:
  `done: task 41, chat`; `proposed: p3/17, to maintainers`; `decided
  first: by ada, accepted, 14:02`; for `adopt`, `README.md`, 4.7's
  words.
- **What a call sends first** (8.2, step 4) prints on stderr, a line
  each: `kept: `, the request's operation, jig's or the forge's (`stop`,
  `adopt`), then its outcome as an act's prints, which names what it
  made or decided. No name and no key: a request's record keeps the
  engine's key, drawn or a name's digest, and the client's, the
  runner's (jig's `protocol/client.md`, section 10), never the name,
  and neither key is of use to an agent.
- **A restored proposal's `kept:` line** prints its identity alone,
  `kept: chat, proposed: t3/17`, with no recipient: jig keeps none with
  the request, so the line is never stale, and the agent reads the
  proposal for its present one. A new call's outcome keeps `, to
  <recipient>`.
- **`adopt`'s `kept:` line** has two forms: `kept: adopt WORDS`, with no
  comma, for an answer in `README.md`, 4.7's words, unknown among them,
  or an answer that could not be decoded; and `kept: adopt, REASON`, as
  every other operation's, for the client's own answers: held, the
  limit, unconfirmed, not kept.
- **What it does not send** prints a line on stderr too: another
  party's kept requests, `kept: N for party P, unsent while it is not
  signed in here` (8.2); the records the client dropped at start,
  `dropped: N kept records these tools cannot read` (5.6).

### 6.2 JSON

`--json` prints the result record instead: one JSON object on stdout,
written with `skein-json`'s writer, the result's fields by their names
in the table, in snake case, numbers as jig's integers, no floats. The
JSON is the contract for scripts; the text follows temper's words, and
may change with them.

- **`help TOOL --json`** prints the tool's record of the table.
- **`--call`'s arguments** are named as the table names them:
  `decide`'s are `proposal`, or `task` and `hold`, then `outcome` and
  `reason`.
- **The `kept:` lines,** under `--json`, are one JSON object a line on
  stderr: `operation` and `outcome`, as the act's; another party's,
  `party` and `unsent`; the dropped, `dropped`.

### 6.3 Exit status

| Status | Meaning | Prints |
|---|---|---|
| 0 | a read answered, an act done or proposed, a wait's condition met | the result |
| 1 | the call failed: usage, the directory refused, its store unreadable (5.6), or busy past the lock's wait (8.3), the engine's address not on loopback, the engine unreachable or the link down past its deadline, signed out, a request past the client's limits, a name the client holds already (`Held`, section 7), unknown past the retention, not kept (DIRECTORY's store not written); the client's own answers alike, a read past its limits among them; a wait's watch ended by the sign-in's end | one line on stderr, `temper ctl: ...` |
| 2 | refused by the engine, and only by it: a key conflict among them (section 7); a wait's subject no longer visible to the party | jig's refusal, typed in JSON, in words in text; the act's alternatives where the refusal names them (read again, propose instead) |
| 3 | a wait's timeout, or its subject's watch ended, its condition not met | the current document, or the last, and its digest |
| 128 + N | ended by signal N, `SIGINT` or `SIGTERM`: 130 or 143 (8.2) | nothing more |

- **2 is the engine's answer alone:** what it decided about the
  request, with the alternatives it names. The client's own answers
  are 1. Two of them never reach a call: unconfirmed, since the runner
  sends nothing before `me` is live (8.2, step 3), and the client's key
  conflict, since each call's keys are its own (section 4). Either is
  the tools' bug: it exits 1, says so, and the guide says to report it
  as a finding (11.5).
- **A call cut by a signal** ends at once and exits 128 plus the
  signal's number, never 0 (8.2).
- **A call whose link stays down** past its deadline exits 1, and what
  it sent stays in the small store, for the next call that sends (8.2).
- **Only `sign-in` creates DIRECTORY.** Three of 1's lines, word for
  word: `temper ctl: DIRECTORY does not exist`, for any other call on a
  DIRECTORY that is not there; `temper ctl: DIRECTORY is not signed
  in`, for a call that needs the token when DIRECTORY holds none; and
  `temper ctl: the engine at ADDRESS did not answer within 30 s`, the
  link down past its deadline (section 12), ADDRESS as `address` holds
  it.
- **`adopt`:** adopted exits 0, with branch protection unknown or not
  (`README.md`, 4.7, prints "Branch protection could not be read."
  then); unknown past the retention exits 1; each other outcome there,
  not found, no access and refused among them, exits 2, in its words.

### 6.4 Bounded

- **Every result has a ceiling,** in rows and bytes: a list's page, the
  activity lines `task` adds, a conversation's page, a turn's part. A
  text cut prints how much was left and the call that reads the rest.
- **Long texts are paged, never whole by default:** `chat`, `activity`
  and `turn` take a cursor or an offset. An agent's context is the
  budget the defaults are chosen for.
- **The numbers** are section 12's. A path is never cut: the tools
  print it whole, within the client's bound on a path's segments
  (`jig-client-domain`'s `Limits`, `segments`); nor is an authority,
  within its bound on an authority's entries.

## 7. Requests: once, and named

- **Through jig's request table.** Every act but one that carries a
  value (below) is a keyed request: `jig-client` keeps it before sending
  it, sends it again under its key until its answer is durable, and
  answers it once (jig's `client/domain/client.md`, section 6).
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
  ask is refused by the engine as a key conflict. The name, not
  DIRECTORY, makes a request the same: a rerun from any directory signed
  in as the party is the same request, and DIRECTORY's store keeps a
  request only until it is answered (8.2). Past the engine's retention,
  which `me` prints, the name is free again, and asking under it is a
  new request.
- **A named request is built from the call and `me` alone,** never from
  the clock, so a rerun builds the same ask: a deadline a template gives
  as a span is sent as that span, which the engine makes a wall when it
  makes the task (jig's `client/domain/contract.md`, 3.2). A rerun
  after `me` changed what the request is built from (a template, a
  bound) is another ask, refused as a key conflict.
- **A name the client holds already** is answered `Held`, nothing sent
  (jig's `client/domain/client.md`, section 6). A call sends what
  DIRECTORY kept, and waits for each answer, before it sends its own
  (8.2), so it meets `Held` only past that, or for a name another
  party's parked request holds (8.2); it exits 1.
- **Names are the party's.** Agents sharing a party give their names a
  prefix of their own; the guide says so (11.5).
- **Acts that decide name the revision the agent saw;** a moved one is
  refused, and the agent reads again. A repeat of a decision is answered
  as decided first, by the party itself.
- **The other acts** (`stop`, `cancel`, `decide`, ...) take `--key`
  optionally; without it the table draws one, which the small store
  keeps until answered (8.2).
- **A request carrying a value,** `add-key`'s and `replace-key`'s (5.5,
  area 5), is sent once and never kept: `jig-client` writes no record of
  it to DIRECTORY's store and never sends it again (jig's
  `client/domain/client.md`, section 6; jig's `protocol/client.md`,
  section 10), and the engine takes the value out as it decodes the
  request, so no committed request or answer holds it (jig's
  `domain/accounts.md`, section 5). The client's own answers *unsent*
  (the engine did not make it) and *lost* (it may have) exit 1 with
  nothing kept, as a call cut or a link down before its answer does, and
  `pending` lists nothing of it. The agent asks again with the same
  `--key` and the same FILE: within the engine's retention, `add-key`
  gets the first answer, never a second account. `replace-key` takes
  `--key` optionally, as the other acts do, but the name it draws is not
  kept, so only a name the agent gives can be asked again; a rerun
  without one replaces the key again.

## 8. The command line: `temper ctl`

### 8.1 Its shape

```
temper ctl DIRECTORY TOOL [ARGUMENT...] [--json]
temper ctl DIRECTORY --call FILE [--json]
temper ctl help [TOOL]
```

- **DIRECTORY** is the client's private directory, as for `temper open`
  (`README.md`, 6.1): its credential, `token`, its small store, `store`,
  and the engine's address, `address`, all `jig-client-shell-native`'s
  or beside them, `0o700` and refused unless the user's own. A
  DIRECTORY is `temper ctl`'s or `temper open`'s, never both's (section 15).
- **`address`** is `temper-client-shell-cli`'s: one line,
  `http://HOST:PORT`, `0o600`, written by `sign-in` and read by every
  other call.
- **`--call FILE`** reads one call as JSON, `{"tool": ..., "arguments":
  {...}}`, from FILE, or from stdin for `-`.
- **`sign-in FILE|-`** signs in by the link `temper serve` printed, read
  from FILE or stdin and never an argument (section 9), on loopback, and
  writes the token and the address the link names, as `temper open`
  does (`README.md`, section 7). `sign-out` ends the sign-in, and jig's
  native shell, keeping none, leaves `token` a zero-byte file, which a
  call reads as no token. Every other tool signs in with the token
  DIRECTORY holds (`SignIn { with: Held }`).
- **Loopback only,** as `temper open`, until the clients' listener
  serves TLS: a call checks that `address` is on loopback, `127.0.0.0/8`
  or `[::1]`, before it sends the token, and otherwise exits 1, "the
  engine's address must be on loopback".
- **No configuration file;** its inputs are its arguments and DIRECTORY.

### 8.2 One call, one process

The host of `jig-client`'s one-shot mode (jig's
`client/domain/client.md`, section 20), ordinary Rust on skein's loop,
as the terminal's host is:

1. parse the arguments against the table; a usage error exits 1 before
   anything is opened;
2. open DIRECTORY and check its address (8.1); for a call that sends,
   take its lock (8.3) and load the small store's records;
3. start the tools' root, sign in with the token, wait for `me`;
4. send what the store kept for `me`'s party, each answered as a line
   on stderr, `kept: ...`, a restored proposal's without its recipient
   (6.1); the step ends once each is answered and its record erased, so
   that the store keeps nothing of the party's when the call is sent;
5. run the call: a read answered, an act's answer durable, a wait's
   condition met or its timeout;
6. print the result on stdout, close the stream, exit with the call's
   status.

- **A termination signal,** `SIGINT` or `SIGTERM`, arrives as skein's
  `Shutdown` (skein's `io.md`, section 7), and ends the call at once:
  it sends nothing more, leaves DIRECTORY's kept requests as they are,
  prints nothing more, and exits 128 plus the signal's number, 130 or
  143. What it sent stays in the small store, the next call that sends
  sends it again under its key, and a rerun under its name gets its
  first answer (section 7).
- **A link down** past the link's deadline (section 12), at step 3, 4
  or 5, exits 1; what was sent stays in the small store, as after a
  signal.
- **Another party's kept requests,** in a DIRECTORY signed in again as
  another party, stay parked and unsent, until their party signs in
  there again or the retention passes and they are answered unknown
  (jig's `client/domain/client.md`, section 11). Step 4 does not wait
  for them; the call says how many (6.1) and goes on.
- **`pending`** lists what the small store keeps (5.6).

### 8.3 Several calls at once

An agent runs a `wait` while it acts, or several agents share a party.

- **A call that sends** takes an exclusive lock on DIRECTORY for its
  run, so two never write the small store over each other: skein's
  `Lock` on a root (skein's `io.md`, section 5), on DIRECTORY itself,
  since `store` is replaced by a rename on each write. A second tries
  again every 100 ms, and past the lock's wait (section 12) exits 1,
  "DIRECTORY is busy with another call". The lock ends with the process,
  however it ends.
- **Which take it:** every act, `sign-in` and `sign-out`; never
  `pending`, `help`, a read or a wait.
- **A read and a wait** take no lock, and hold their client's store in
  memory, empty, never DIRECTORY's file: they write no record and send
  nothing but reads and watches. A client loaded from DIRECTORY's
  records would send the stream they name gone (jig's
  `client/domain/client.md`, section 11), ending the watches of the
  call that opened it.
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
- **Credentials are files,** private, never arguments, never printed;
  the printed link too, which `sign-in` reads from a file or stdin.
- **The agent's LLM is not the deployment's.** The LLM the agent itself
  runs on, to drive `temper ctl`, is its harness's. The deployment's
  runs are paid by jig's LLM accounts, added from the client
  (`../protocol/credentials.md`, section 3): on a box, by the deployment's
  key the harness adds (section 10), which pays for every party's runs
  (jig's `domain/accounts.md`, 4.2). An agent acting for a person
  connects that person's own accounts with area 5's tools, as the
  person would on the accounts page.

## 10. The box

The deployment an agent drives is an owned setup run by `temper box`
(`../protocol/setup.md`, section 7), in a scratch directory of the QA
harness's:

- **Started:** `temper box CONFIG.json`; its stdout carries `temper
  serve`'s sign-in links, and the harness passes the first to `temper
  ctl DIRECTORY sign-in -` on stdin.
- **The deployment's LLM key:** a fresh box has no LLM account, so no
  run starts (`../protocol/credentials.md`, section 3). Once signed in, the
  harness adds one before the agent's first act: `add-key PROVIDER FILE
  --deployment --key box-llm`, from a private file of its own, then
  `wait accounts --until usable` (5.4). The key is QA's own, with limits
  of its own (`../draft/qa.md`, section 7); the agent is never handed its
  file, and the harness removes the file with the box. On the process
  tier and in the real loop, it is a key skein's fake LLM peer accepts
  (`../domain/testing.md`, 2.4 and 2.6).
- **The person on Forgejo:** the harness plays the person of
  `../protocol/setup.md`, 6.5: it signs in on Forgejo's web with
  `person_password` and changes it, and makes the repository to adopt
  in an org, which the agent adopts with `adopt` as the first act of
  area 1.
- **The forge read directly:** its API and `git` on `owned.listen`, with
  a token the harness makes for the person, as a person makes one
  (`../protocol/setup.md`, section 16), whose scopes are reads alone:
  `read:repository`, `read:issue`, `read:organization` and `read:user`.
  The agent is handed that token, never the preparation's
  `admin_token`, which reconcile alone reads (`../protocol/setup.md`,
  section 9). Forgejo refuses the agent's writes with it; the agent
  changes the forge only through temper, and the guide says so (11.5).
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
- **Digests and conditions:** a digest moves with a document's change
  and only then; each condition over documents that meet and miss it.
- **The boundary:** `src/jig/` names no temper item (section 4).
- **The bounds:** each of the tools' ceilings within the client's limit
  it reads from (section 12).

### 11.2 `tests/client`

`temper-client-world` runs the tools' root against temper's engine root,
beside the fake person, as a scripted agent making calls (`README.md`,
section 10; `../domain/testing.md`, 6.3). Area 1's stories: a chat started, its
reply waited for, a message, a turn too large for a page read whole,
the chat closed; a repository adopted; a held task released. A turn
too large for a page exists only where the engine's bound on a turn
passes its page bound, so the world's profile sets it so. The world's
faults hold: a call cut by a link drop or an engine restart, asked
again under its named key, is made once. Each area adds its
stories. Area 5's, over the engine root's accounts child with each
source's mechanism scripted: a key added, then the deployment's,
`accounts` showing both with no value in DIRECTORY's store or in any
document the tools print; an `add-key` cut before its answer and asked
again with its key and file, one account made; a sign-in started, its
answer scripted made, refused and past its deadline, each meeting
`wait accounts --until settled`; a limited notice, the next task
waiting for its account and `task N` saying why, then `retry` and the
run started; `rank` and
`forget`; a program source declared, and `verify` answered as each host
reports.

### 11.3 The real loop

`tests/real-loop` hosts the tools' root over jig's native transport,
the standalone engine and worker, smith's agents on skein's fake LLM
peer's scripted turns, skein's forge peer hosting the fake forge, and
the fake git program over that forge, on one thread and one real ring
(`../domain/testing.md`, 2.4). Agents are hosted in that loop and
joined to their worker by real pipes. The store and client credentials
use a scratch directory on the real filesystem.

Its stories sign in, start a chat and wait for its reply, and satisfy one
client's wait by another client's act; each drains with everything
settled. Then a service restart retains the work. A call whose answer was
lost is made once: the tools' root is dropped once the engine holds its
request, before its answer, and a fresh one, on another DIRECTORY signed
in as the same party, asks under the same name and is answered the first
answer; the party has one chat. A DIRECTORY whose store holds a record of
a store version the tools do not read: `pending` counts it, and the next
call that sends says it dropped it. One signed in again as another party:
the first party's kept request stays unsent, and the call goes on. The
forge's stories adopt a repository, and clone, push and land through the
fake git program on the shared in-memory fake forge, with CI by the fake
forge's content marker and its webhooks delivered to the engine
(`../domain/testing.md`, 2.4). Real git meets a forge in the process
tier (11.4). The forge's stories run on `skein-world`'s
`World::new_controlled`, unchecked: fake git's memory in the shared fake
forge outlives the git program that made it, which the checked world
fails (skein's `testing-strategy.md`, section 6; skein's
`fake-forge.md`, section 5). Every other story here is checked.

Until area 5's tools (5.5), the deployment's credential is handed in as
a `handed_in` file of the configuration the test writes, the engine's
handed-in account (`../domain/testing.md`, 2.4). With area 5, every story
first adds a key the fake LLM peer accepts, `add-key` from a scratch
file, as section 10's harness does, and waits for `usable`. Area 5's
stories (`../domain/testing.md`, 2.4):

- **A sign-in:** `connect` against skein's fake issuer, the browser's
  visit and the redirect to the engine's loopback listener driven by the
  test, then `wait accounts --until settled`, the account usable, and a
  run on it whose request the peer observes with its bearer.
- **A limit:** a scripted turn answering with the provider's exhausted,
  with a reset, makes the source limited, and the next task waits for
  its account (jig's `domain/authority.md`, 8.3); `retry`, then `wait
  task N --until phase:running`.
- **A program's login:** `declare`, with smith's fake `claude` as the
  agent's program, its login signed in in the source's directory under
  the agent's programs directory; `verify`, then `wait accounts --until
  settled`, the source usable on the worker's host, and a run on it
  through the program backend.
- **No value kept:** after each, neither DIRECTORY's store nor the
  engine's store holds a key's or a record's value in a request, an
  answer or a fact; the secrets' store alone does.

Each call of `temper ctl` in these stories is its host,
`temper-client-shell-cli`'s, hosted as a program of the real world, one
per call, on its DIRECTORY: the lock, DIRECTORY's files and the call's
exit run on the real kernel. Two acts at once on one DIRECTORY are
serialized by the lock: skein's `Lock` is `flock`, held per open
descriptor (skein's `kernel.md`), so two programs of one process contend
as two processes do.

These are a small set of focused tests at the gate. They reuse lower
worlds' scripts and outside expectations where applicable, and add
checks of the full transport, persistence and teardown on the real
kernel. Most tool behaviour stays in `tests/client`; the harness checks
memory ownership, outstanding operations and release between iterations.
A real-loop failure is kept with its trace and reproduced lower down
where possible; its seed does not replay the kernel's schedule.

### 11.4 The process tier

`temper serve` and `temper ctl` as processes over skein's fake LLM peer
(`../domain/testing.md`, 2.6), on a scratch directory of their own, for
what needs the executable: argv through `main`, printed output, the OS's
exit statuses, the printed link read on stdin (`sign-in -`), the
deployment's credential (a `handed_in` file until area 5's tools, as in
11.3; with them, its key added from a private file as the first act,
section 10, and a key's file refused when it is not private), an act
cut by a termination signal, its status 128 plus the signal's number
(8.2), then asked again under its key, made once, and agent process
launch. One landing on the owned Forgejo proves a change against real
git and CI (`../domain/testing.md`, 2.6). The lock, DIRECTORY's files
and a call's exit are the real loop's (11.3).

### 11.5 The agents' guide

`temper ctl help` prints a short guide, then the tools from the table:
how to sign in, the loop of 5.4, naming keys and their prefix, reading
acts, the exit statuses, reading the forge and never changing it but
through temper (section 10), and an LLM account's key read from a
private file, never typed, with a sign-in followed by `wait accounts`
(5.5, area 5). The guide is text of `temper-client-tools`; a
test fails when it names a tool the table lacks.

### 11.6 Findings into stories

Each bug an agent finds through the tools ends as a story in
the lowest tier that shows it, before its fix lands: a domain world,
`tests/client`, the simulated world, the real loop or the process tier
(`../draft/qa.md`, section 1). A story's calls are the tools' calls, so an
agent's history of `temper ctl` commands transcribes into one.

## 12. The bounds

Four bounds decide how much of a document a call prints:

| Bound | Whose | It decides | Past it |
|---|---|---|---|
| a page's bytes | the engine's (jig's `client/domain/contract.md`, 5.1) | whether a turn comes in a page by its bytes or by its size | the turn's line, its size and the call that reads it (5.2) |
| one read's bytes | the engine's (the read of one turn, jig's `client/domain/contract.md`, section 6) | whether `turn` is answered whole | its size, and that it is past the limit |
| one document's bytes | the client's, `document_bytes` | whether the client takes a document whole | a turn: its size, and that it is past the limit; any other document: the client's `Limit`, exit 1 |
| the tools' window | the tools' | how much of a result prints | the rest by its cursor, or `--part` and `--from` (6.4) |

- **A turn past either read bound** answers exit 0, with its size and
  "past the limit; report it": the agent reports it as a finding, and
  never works around it.
- **The tools' window is within the client's limit:** a step test fails
  when one of the tools' ceilings passes the client limit it reads from
  (11.1).

The numbers are constants of `temper-client-tools`' `limits` module. The
client's are the tools' own preset of `jig-client`'s `Limits`, not the
terminal's, whose window shows two rows:

| What | Bound |
|---|---|
| rows: a list's page, the inbox's needs and updates together, the activity lines `task` adds, a conversation's page; the client's rows per window | 32, the most jig's client wire takes in a limit of rows (its schemas' ceilings; jig's `protocol/client.md`, section 3) |
| a result's text | 32 KiB |
| a turn's part | 16 KiB |
| one document, the client's `document_bytes` | 1 MiB |
| a path's segments; an authority's entries | 9; 12, as the terminal's (`README.md`, 5.4) |
| the link's deadline: the engine unreachable, or a call's request or answer unheard | 30 s |
| the lock's wait | 60 s, tried every 100 ms |
| a key's file, less a final newline (5.5, area 5) | 4 KiB; past it, or empty, exit 1 with nothing sent; a key the engine refuses is its refusal, exit 2 |
| a wait's timeout | 300 s by default, at most 3600 (5.4) |

The client's other limits are the terminal's (`README.md`, section 2).

## 13. What it takes of jig

The tools work around nothing of jig's and skein's. Besides the
contract's reads, watches and requests, they take:

- **A named key:** `jig-client`'s `Send` under a key the application
  names, refused `Held` when the client holds it already (jig's
  `client/domain/client.md`, section 6), on skein's request table's
  given keys (skein's `lib.md`, section 12); everything else of keyed
  requests is the contract's (jig's `client/domain/contract.md`, 3.1).
- **A read of one turn,** `turn{n, attempt, turn}` (jig's
  `client/domain/contract.md`, section 6), answered from the store's
  `Scope::Turn` (jig's `protocol/store.md`, 7.3); on a page, a turn past
  the page's bound keeps its size (jig's `client/domain/contract.md`,
  5.1).
- **A deadline as a span** in a request a template gives one (jig's
  `client/domain/contract.md`, 3.2), so that a named request is built
  without the clock (section 7).
- **For area 5,** jig's accounts (jig's `domain/accounts.md`, section 5):
  their requests (jig's `client/domain/contract.md`, 3.2), each under a
  key as the others, a key's bytes held to the tools' own bound
  (section 12) and the engine's refusal past its own exit 2; the
  `accounts` watch (jig's `client/domain/contract.md`, 7.1); the models
  a party may use, in `me` (jig's `client/domain/contract.md`, 5.1),
  and the LLM choice in a task's request (jig's `domain/tasks.md`,
  section 3); and a request carrying a value,
  sent once and never kept nor resent (jig's `client/domain/client.md`,
  section 6; jig's `protocol/client.md`, section 10).
- **The rest as jig designs it:** its helpers (`shapes`, `facts`,
  `thread`, `words`; jig's `client/domain/client.md`, section 19), the
  documents' acts and `me`'s bounds (jig's `client/domain/contract.md`,
  5.3), the client's bound on a path's segments, a service's sign-in,
  and the one-shot mode, whose modes "differ in the host alone" (jig's
  `client/domain/client.md`, section 20).

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
  forge's parts (`forge.md`, section 9).
- **A service's credential reissued, or a service removed** (jig's
  `domain/people.md`, section 3): tools of area 3.
- **An account moved,** `export ACCOUNT FILE` and `import FILE` (keyed),
  of area 5, with jig's move (jig's `domain/accounts.md`, section 6): the
  record written to a private file, refused if the file exists, and read
  back from one, `import`'s request never kept (section 7). `move` and
  `take` are area 3's names.
- **An account's uses** set, as a tool of area 5, when the QA loop asks
  for one (section 15).

## 15. Open questions

- **`temper open` and `temper ctl` on one DIRECTORY:** not supported and
  not detected (`README.md`, section 11). Default: the lock of 8.3 is
  `temper ctl`'s alone, until `temper open` takes it.
- **The one-shot host's place:** temper's, in `temper-client-shell-cli`.
  Default: it moves into `jig-client-shell-native` with the extraction
  (section 14), if jig wants one.
- **Naming a source** in area 5's `--source S`: by its kind, which
  names one source while an account holds one of each kind. Default:
  by kind, and by a program source's name once an account may hold two
  of a kind.
- **An account's uses:** no tool, each account taking jig's defaults
  (jig's `domain/accounts.md`, 4.2). Default: a tool of area 5 once the
  QA loop plays a service paid by a person's account (section 14).
