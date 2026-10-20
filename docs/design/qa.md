# QA in the lab

2026-10-20, with director decision 253. temper's binding to skein's lab (skein's
`lab.md`): the experiments temper keeps in `lab/`, the cluster, seed,
charter, tool and checks each runs with, what the fakes' scripts must
say, the cost classes they name, and the first campaign's process. It
settles M1 of `draft/qa-loop.md`: a person's chat asks for a one-file
change on `fake/widget`, which is proposed, pushed, passes CI and its
review, and lands. The lab's records, commands, outcomes, verdicts and
findings are skein's `lab.md`'s, cited by section and never restated.
What M1 waits for, and its adopted decisions, are section 11.

## 1. In one page

- **One story, area 1** (`client/tools.md`, 5.5): a person, played by
  Codex, asks a chat in project 1 for one new file on `fake/widget`; the
  chat delegates the change, the change is pushed, passes CI by its
  content and an agent's review, and lands. The person follows it
  through `temper ctl` alone (section 2).
- **Three experiments in `lab/`:** the story with its driver (H); the
  same cluster with the story scripted and no driver, the free smoke
  (S); H with `temper serve` killed after the driver's first action
  (P). P with its fault and planted list cleared is H byte for byte,
  so H's verdict is P′'s
  (section 3).
- **Real binaries on fakes:** `temper fake forge`, then `temper fake
  llm`, then `temper serve` on documents rendered from their ready
  lines; the worker's git is the subject's binary under the name `git`
  (section 4).
- **A trusted seed with run credentials only:** sign in by serve's
  printed link, add the fake model's key, wait for it and for the
  forge, adopt `fake/widget` (section 5).
- **The driver reaches temper only through `ctl`,** a wrapper that
  fixes the client's directory, pins serve's address and takes area 1's
  tools but `sign-in`, `sign-out` and `--call` (section 6).
- **The fakes speak real protocol by script:** a chat, a coding and a
  review conversation in skein's script vocabulary, and CI by the fake
  forge's content cue (section 7).
- **Checks from outside:** the runner's own; the forge's promises by
  `temper fake forge check`; the story's end, read from the fake's
  observations and store: landed on `main`, `NOTICE` exactly as asked,
  nothing else changed (section 8).
- **The planted kill** gives a stable stop signature, filed, reproduced,
  classed `experiment` and closed by H's verdict (section 9).
- **Eight paid driver runs planned of fifteen:** one calibration, three
  healthy, three planted and one rerun (section 10).

## 2. What M1 is, and is not

- **For:** temper end to end on real processes (`draft/qa-loop.md`,
  section 2): `temper serve`'s start on a rendered document, the engine
  and its worker in one process, `temper agent` processes, `temper fake
  git` as the worker's git across processes, `temper ctl` as an agent
  meets it, the forge's webhook deliveries across a network, and
  the loop from a chat's request to a landing.
- **Area 1 alone:** a chat to a landed change. No goal, plan, approval,
  second party, account or forge act by the agent.
- **Not the box, `temper init`, the demo or the first run:**
  `application.demo` is null and no step of the box runs; those are
  experiments of their own, later (`draft/qa-loop.md`, section 12).
- **Every product process is the subject's `temper`:** `serve`,
  `agent`, `ctl`, `fake git`, `fake llm` and `fake forge`. The fake LLM
  speaks a provider's protocol to temper's agents; it is not the driver,
  which is Codex on the lab's side (skein's `lab.md`, 6.1).
- **Deliveries are required.** M1's story runs on the fake forge's
  webhook deliveries to serve across processes (11.2). Polling stays
  only where temper already uses it as its own fallback. The lab adds
  no polling substitute for deliveries; the missing delivery and bound
  webhook-address interfaces are named waits (11.1).

## 3. The experiments

### 3.1 Three directories

| Directory | `name` | Driver | Fault | For |
|---|---|---|---|---|
| `lab/chat-change/` | `chat-change` | the charter of 6.1 | none | H: the story, judged on three seeds |
| `lab/chat-change-smoke/` | `chat-change-smoke` | null | none | S: H's cluster, seed and checks, with the story as one more seed action (5.2) |
| `lab/chat-change-killed/` | `chat-change` | as H | serve killed (section 9) | P: the planted fault |

- **H's files:** `experiment.json` (skein's `lab.md`, 4.1), `charter.md`
  (6.1), the templates of 4.4, `tools/ctl` (6.2), `checks/end-state`
  (8.4) and `NOTICE` (3.3). S adds `smoke/story` (5.2).
- **P is H with a fault:** its files are H's, byte for byte, but
  `experiment.json`, which differs by its `faults` and `planted` alone;
  its `name` is H's. So P with `faults` and `planted` cleared has H's tree and digest
  (section 9). Two directories hold them because both run at one head.

### 3.2 The subject

`lab subject --repository <temper> --commit C --exclude lab --artifact
temper=<binary> --artifact git=<the same binary>` (skein's `lab.md`,
10.2).

- **Two artifacts, one file:** `temper` runs every product process and
  command; `git` is the same binary under the name `git`, which makes
  it `temper fake git` by its `argv[0]` (`domain/testing.md`, 4.3) and
  is the worker's `git`, an absolute path. The lab's store holds no link
  (skein's `lab.md`, 2.2), where a machine would use one.
- **`lab/` is excluded,** so the driver never reads the experiments,
  their scripts or their checks (skein's `lab.md`, 6.2). It may read the
  rest of temper's source and documents (`draft/qa-loop.md`, section 2).

### 3.3 The change

- **Asked for in these words:** `Add a file NOTICE to widget holding
  one line: Maintained by temper.`
- **Before:** `fake/widget`'s `main` as the fake seeds it
  (`protocol/setup.md`, 8.4), with no `NOTICE`.
- **After:** `main` one squash merge further (skein's `fake-forge.md`,
  2.2), whose tree is its parent's with `NOTICE` added, 22 bytes,
  `Maintained by temper.` and a newline, and nothing else added, removed
  or changed. The experiment's `NOTICE` holds those bytes: the coding
  script writes them (7.1) and the end-state check compares with them
  (8.4).
- **A new file,** so that the before is exact whatever else the fake
  seeds, which is `temper-fake`'s content (`protocol/setup.md`, 8.4).

## 4. The cluster

### 4.1 Its order

```
forge    temper fake forge, from its document, port 0          ready "listening " → address
└─ llm   temper fake llm; its scripts name the forge's address  ready "listening " → address
   └─ serve  temper serve, its documents naming both            ready → link and bound webhook address
             then the seed actions (section 5), then the driver
```

- **One chain.** The chat's `delegate` names the forge's resources by
  the forge's address (7.1), so the model starts after the forge;
  serve's documents name both. Each start has one candidate, so the
  start-order draw changes nothing (skein's `lab.md`, 5.4).
- **Addresses from ready lines alone** (skein's `lab.md`, 4.4): each fake
  prints `--ready`'s prefix and its bound address (`protocol/setup.md`,
  8.4). Serve must report its bound webhook address, a pending CLI
  interface routed through the director (11.1). Its sign-in link,
  `http://HOST:PORT/jig/v1/sign-in/link?secret=…` (`client/README.md`,
  section 7), gives the clients' listener's bound address. Reconcile
  registers the webhook after serve binds; the fake delivers to that
  hook. The bound-address notification and bootstrap channel's exact
  syntax remain their owner's, not invented in these templates.

### 4.2 Its processes

```
forge  files  forge.json
       argv   {artifact.temper} fake forge --document {proc.forge.dir}/forge.json
              --ready "listening " --seed {seed.forge} --observations {proc.forge.dir}/observations.jsonl
llm    after  forge
       files  llm.json
       argv   {artifact.temper} fake llm --document {proc.llm.dir}/llm.json
              --ready "listening " --seed {seed.llm}
serve  after  llm
       files  config/temper.json, config/agent.json, config/charters/chat.txt,
              config/charters/coding.txt, secrets/worker.secret, secrets/worker.digest
       argv   {artifact.temper} serve --config {proc.serve.dir}/config --data {dir.data} --cache {dir.cache}
```

- **`dirs`:** `client`, `data` and `cache` (skein's `lab.md`, 4.1).
- **`env` is `{}`** for each: nothing of the runner's or the machine's,
  every path given.
- **`memory_bytes`:** null for the fakes, which declare none; for
  serve, what its document states for the engine and its worker, which
  share its process (`protocol/configuration.md`, section 7).
- **Files beneath a directory** (`config/`, `secrets/`, `setup/`) keep
  the configuration directory free of secrets
  (`protocol/configuration.md`, section 12) and the fake's setup
  directory to its handover alone (`protocol/setup.md`, section 3); the
  lab refuses only an absolute or climbing path (skein's `lab.md`, 4.7).

### 4.3 Run credentials

| Credential | Made from | Where | Read by |
|---|---|---|---|
| the fake model's key | the fake LLM from its decimal u64 `--seed` | fake-written 0600 output, expected at `{proc.llm.dir}/llm.key`; exact output interface pending (11.1) | the fake LLM; the seed's `add-key` |
| the fake forge's admin token | the fake forge from its decimal u64 `--seed` | fake-written 0600 output, expected at `{proc.forge.dir}/setup/admin_token`; exact output interface pending | the fake forge; serve's setup host, once, which removes it (`protocol/setup.md`, section 3) |
| the sign-in link | serve | its ready line | the seed's `sign-in`, which spends it (`client/README.md`, 6.7) |
| the owner's client token | the sign-in | `{dir.client}` | the seed's acts; the driver's `ctl` |
| the bot's tokens | serve's reconcile on the fake | jig's secrets' store in `{dir.data}` | serve |
| the worker's secret and digest | a fixed pair of the experiment's, as the process tier's | `{proc.serve.dir}/secrets/` | serve |

- **No machine secret:** no file, argument or environment of the
  experiment names the home directory, `CODEX_HOME`, a real forge or a
  real provider. Each credential above opens only this run's loopback
  processes, which end with it.
- **The runner invents no fake credential bytes.** It passes each
  fake's decimal u64 seed and names its private output files; deriving
  and writing the key or token is the fake's. This is decision 253's
  default, routed to first-run; its final encoding and output interface
  remain a wait. The outputs exist before readiness and are not runner
  file templates.
- **Recorded seeds and captures:** draws are in `run.json`'s `schedule`
  and captures in its processes (skein's `lab.md`, 5.4 and 5.6).
  Credential derivation belongs to the recorded fake executable's
  identity. The link is spent by the seed before the driver starts.
- **Nothing reaches the driver** but through `ctl`: its own commands
  read none of these files (6.2).

### 4.4 The documents

- **`forge.json`,** the fake forge's lab form (`protocol/setup.md`,
  8.4): `listen` `127.0.0.1:0`; the private admin-token output path above
  through the pending fake-owned output interface;
  `owner` `fake`; `repositories`, `widget` landing on `main`; and the
  CI cue of 7.2, a capability the form lacks (11.1).
- **`llm.json`,** the fake LLM's lab form: `listen` `127.0.0.1:0`; the
  private key output path above through the pending fake-owned output
  interface; the provider's one model with its window and
  output; the scripts of 7.1.
- **`temper.json`,** serve's document, from the real loop's landing
  fixture (`tests/real-loop/src/landing.rs`, `fixture` and `configure`)
  and the process tier's documents, with:
  - project 1's landing rule on `fake/widget`'s `main` asking for CI, up
    to date and gate 1, an agent's review, with no approval; its
    owner's authority, the rules' ceiling and its projections on the
    forge's resources, `["forge", "{proc.forge.ready.address}"]` up to
    the owner `fake`;
  - the charters chat, 1, and coding, 2, the owned example's texts
    (`protocol/setup.md`, 8.1), coding granting `inspect`, `modify` and
    `shell`, neither text holding a cue of 7.1;
  - one LLM provider, 1, OpenAI's dialect on its public route at
    `{proc.llm.ready.address}`, its source a key, over TLS with skein's
    test root (11.2);
  - forge 1, Forgejo, its members its setup's: `api`
    `{proc.forge.ready.address}`, plaintext, `git` `http://` and that
    address; its `setup` entry fake, `directory`
    `{proc.forge.dir}/setup`, bot `temper`, `listen` that address, owner
    `fake`, the one repository;
  - one worker in the process, `root` `workspaces`, `git`
    `{artifact.git}`, its agent `{artifact.temper}` with `["agent",
    "{proc.serve.dir}/config/agent.json"]`; the worker's secret and
    digest by their absolute paths;
  - a webhook listener on `127.0.0.1:0`, its actual bound address made
    available to reconcile through the pending serve interface;
    the clients' listener `127.0.0.1:0`, no bundle; `demo` null.
    Keep temper's existing polling fallback; add no experiment polling
    loop or shortened polling schedule as a delivery substitute.
- **`agent.json`,** smith's agent configuration: the landing fixture's
  profile, and the fake LLM's endpoint and model, trusting the same
  root.

## 5. The seed actions

### 5.1 H's and P's

`temper ctl add-key` is available at temper `dfc7cc70a`, in the
`add-key PROVIDER FILE --deployment --key KEY` form used below. Its
file must be private, regular and owned by the caller; the host strips
one final newline. The fake's key output remains first-run's pending
interface, not a runner-generated value.

Each is `{artifact.temper} ctl {dir.client}` and then:

| Action | Arguments | For |
|---|---|---|
| `sign-in` | `sign-in -`, stdin `{proc.serve.ready.link}` and a newline | the deployment's owner signs in by the full printed link |
| `add-key` | `add-key 1 {proc.llm.dir}/llm.key --deployment --key lab-llm` | the fake model's key, the deployment's account (`client/tools.md`, 5.5, area 5) |
| `usable` | `wait accounts --until usable --timeout 100` | the account usable |
| `forge` | `wait forges --until connected:1 --timeout 100` | reconcile on the fake done (`protocol/setup.md`, 8.4) |
| `adopt` | `adopt --project 1 --forge 1 --owner fake --name widget --landing main --key lab-adopt` | adopted with CI on, at the form's defaults (`client/tools.md`, 5.3) |

- **Each must exit 0,** or the run fails `action/<name>/exit:<status>`
  (skein's `lab.md`, 7.1). Each wait's timeout is below
  `command_seconds`, so a wait not met fails by its own exit, 3.
- **No sign-out:** `{dir.client}` stays signed in as the owner, and is
  the driver's through `ctl`. Every seed request has been answered, so
  the driver's first call sends nothing kept (`client/tools.md`, 8.2).
- **The box's harness's acts** (`client/tools.md`, section 10), with no
  credential handed in.

### 5.2 S's story

S adds one action, `story`: `/bin/sh {experiment}/smoke/story` with
`ctl`'s argument prefix (6.2) after it. It makes the person's calls
through the wrapper: `chat --project 1 --key smoke-chat` with the words
of 3.3; the chat's number from `done: task N, chat`; `wait task N
--until ended --timeout 110`; `task N --tree`. It exits 0 only when the
chat ended, which it does after its delegate's result (7.1).

## 6. The driver

### 6.1 The charter

`charter.md`, whole:

```
You are Ada. You own this temper deployment; its project 1 has adopted
the repository fake/widget, which lands on main. You reach temper through
one tool, ctl: its args are a temper ctl call after its directory, such
as ["me"] or ["chat", "--project", "1", "--key", "notice", "..."], and
["help"] prints temper's guide for agents.

Your goal: ask a chat in project 1 for this change, in exactly these
words, then follow it until you can tell whether it landed:

    Add a file NOTICE to widget holding one line: Maintained by temper.

It has landed when temper's documents show the change's task done and
its change landed. Read that in temper; the forge is not yours to read
here.

Do not adopt repositories, add accounts or connect forges, and do not
look for another way to reach temper, the forge or the model. Your
commands may read temper's source and write your work directory only.

End with the lab's report: done only if you saw the change landed.
```

### 6.2 The `ctl` tool

```
{"name": "ctl",
 "argv": ["/bin/sh", "{experiment}/tools/ctl", "{artifact.temper}", "{dir.client}", "{proc.serve.ready.link}"],
 "env": {}, "writes": ["{dir.client}"], "reads": ["{experiment}/tools/ctl", "{artifact.temper}"]}
```

`tools/ctl` is POSIX sh, with the shell's builtins alone, then `exec`.
For each call it checks, in order:

1. **The tool is area 1's** (`client/tools.md`, 5.5): the reads `me`,
   `inbox`, `tasks`, `task`, `chat`, `activity`, `turn` and `run`;
   `wait`; the acts `chat`, `message`, `close`, `adopt`, `answer`,
   `decide`, `release`, `stop` and `cancel`; and the host's `pending`
   and `help`. Refused: `sign-in` and `sign-out`, which replace or drop
   the directory's credential and the address it names
   (`client/tools.md`, 8.1); and every
   tool of areas 2 to 6, outside the story and where accounts, keys,
   forges and pasted files are named.
2. **No argument is `--call`** or begins `--call=`: a call read from JSON
   names its tool where the wrapper does not look.
3. **The directory's `address` is `http://` and the link's authority:**
   the engine the seed signed in to, pinned before every call.

Then it runs `{artifact.temper} ctl DIRECTORY ARGS...`, `help [TOOL]`
as `temper ctl help [TOOL]`, and the call's stdin, stdout, stderr and
status are the tool's.

- **A refusal** exits 1 with `ctl: refused: <why>` on stderr, sending
  nothing. It is the experiment's, never temper's, and the transcript
  shows it.
- **The wrapper records nothing:** the lab's server records every call
  before its result reaches the agent (skein's `lab.md`, 6.3). It adds no
  tool: temper's public `temper ctl` is the tool.
- **The boundary is the lab's** (skein's `lab.md`, 6.2): the agent's own
  commands read the source and write its work directory, with no
  network; the tool's view writes `{dir.client}` alone. The agent reads
  no token, run credential, process file or host login, and edits no
  product or process file.

### 6.3 Its report and bounds

- **The report** is the lab's (skein's `lab.md`, 6.4); `done` true only
  for a landing the agent saw in temper's documents.
- **Bounds** are the cost class's (10.1), enforced as skein's `lab.md`,
  6.5 says. That the host's inventory is checked and that usage is kept
  when a drive is cut are the runner's to show; this binding assumes
  them and proves nothing of them.

## 7. The fakes' scripts

### 7.1 The model

Three conversations, each a script cued by its system text, which holds
the charter's instructions and the task's words (skein's `fake-llm.md`,
2.4), and a reply for anything else:

| Conversation | Cue | Turns |
|---|---|---|
| chat | `Add a file NOTICE to widget`, the person's words | 1. `delegate`: one change for the forge's procedure, executor procedure 2, its words `@coding NOTICE`, its parameters `fake/widget`'s number on the fake and the branch `main`, its contract a change, its authority the forge's grants on `["forge", "{proc.forge.ready.address}"]` up to `fake`; 2. the text `I asked for the change; its task follows it.` and `wait`; 3. `finish`, a report: `The change's task has ended; it says how.` |
| coding | `@coding NOTICE`, the delegate's words | 1. `write`, `NOTICE` with the bytes of 3.3; 2. `shell`, `test -f NOTICE && grep -q Maintained NOTICE`, in the shell subset (`domain/testing.md`, 4.3); 3. `finish`, a change titled `Add NOTICE`, its body `Adds NOTICE, as asked.` |
| review | `Review this change at the named head`, the engine's words for a review | 1. `finish`, a verdict, choice 1, `Checked` |
| otherwise | none | the text: `This model makes one change. Ask a chat for it in these words: Add a file NOTICE to widget holding one line: Maintained by temper.` |

- **The real loop's landing scripts are the model**
  (`tests/real-loop/src/landing.rs`, `scripts`): their delegate and
  finish shapes and token counts, with this story's cues, words, path
  and grants. Its coding script's read and edit become a new-file
  write and a shell check here. `write` and `shell` take smith's
  arguments (smith's `domain/tools.md`, section 3).
- **No line claims an outcome.** Scripts answer by turn count, not
  content, so the chat's words say nothing of a landing; a change's
  result names what the worker's push and the forge settled, never the
  model's fields (`domain/agent.md`, 4.4). A chat woken early, by its
  person's words, has its `finish` refused while its delegate lives
  (jig's `domain/tasks.md`, "A task ends after its delegates"), and past
  its last turn answers "done"; the change goes on. What landed is the
  checks' to say (section 8).
- **Needed of the fake LLM's lab form** (11.1): cued scripts whose turns
  hold text lines and tool calls, each call's name and JSON arguments,
  with a finish and a token count per turn: skein's `Script`, `Turn`,
  `Line::Text`, `Line::Call` and `Finish` (skein's `fake-llm.md`, section
  2), where the form's `asked` and `otherwise` are text today
  (`protocol/setup.md`, 8.4). No hold, failing line, menu, chance or
  latency. What a run draws (the forge's address) is rendered into the
  document before the fake starts, so the fake reads fixed scripts.

### 7.2 The forge

- **One repository,** `fake/widget`, landing on `main`, seeded by
  `temper-fake` without `NOTICE`; its number, which the delegate names,
  is `temper-fake`'s.
- **CI follows content:** context `check`, the domain's `Cue { path:
  NOTICE, green: "Maintained by temper." }` (skein's `fake-forge.md`,
  2.3): a head whose `NOTICE` holds that line passes, any other fails.
  The lab form's CI passes every head (`protocol/setup.md`, 8.4); the
  cue per repository is needed of it (11.1).
- **No fault, latency or rate limit** in H, S or P.
- **Its observations** to the file `--observations` names, in
  first-run's encoding of the domain's observations (8.3, 8.4).
- **Its store,** read by the git route's `objects` call (skein's
  `fake-forge.md`, section 4), with no credential and none of the
  domain's faults: the end-state check's content (8.4).
- **Deliveries required:** the fake delivers its forge events to the
  hook reconcile registers on serve's actual bound webhook address.
  Delivery across processes is first-run's pending capability (11.1).

## 8. Checks

### 8.1 The runner's

Stops, panics, survivors, memory, bounds, seed actions, the report's use
failures and planted signatures (skein's `lab.md`, 7.1), with nothing
discounted (skein's `lab.md`, 9.4). The experiment adds no generic check.

### 8.2 Collection

First obtain the fake's complete flush boundary and explicit loss count
while it is running, through first-run's pending interface (11.1).
Its command and encoding remain the fake's contract, not an invented
lab command. Then copy that complete evidence:

`observations`: `/bin/cp {proc.forge.dir}/observations.jsonl
{collect.dir}/observations.jsonl`, with `scan` empty: the file is
copied out of scratch so that the checks and findings read evidence
(skein's `lab.md`, 3.3 and 4.5).

### 8.3 The forge's promises

`promises` uses `{artifact.temper} fake forge check` (`protocol/setup.md`,
8.4), first-run's wrapper of the github owner's landed judge
(`skein_fake_forge_domain::promises`, skein `4f13123c`, `fake-forge.md`,
2.5). It judges once, forced, foreign and order, under skein's `lab.md`,
7.2; its signatures are `promise/<promise>/<subject>`. The experiment
judges no promise of its own.

Its inputs include `{collected.observations}/observations.jsonl`, a
client `Profile` and independent `Decision` records naming repository,
pull and exact head, with their completeness declared even when empty.
The decisions come from temper's own record, never from the merge
being judged (decision 259). First-run designs those inputs and the
final command's argv, with github's review; these are named waits,
not schemas or flags the lab invents.

The judge keeps breaches even when other evidence is undecided. The
wrapper preserves both through the existing check contract (259):

- any breaches: status 1, their `fail` lines, plus an `undecided` line
  when the judge has an incomplete-evidence reason;
- no breaches and undecided: status 2, `undecided` lines only;
- pass: status 0, no stdout.

Loss or unknown completeness never gives a pass. Status 2 with a
`fail` line would contradict skein's `lab.md`, 7.2.

### 8.4 The story's end

`end-state`: `{experiment}/checks/end-state
{collected.observations}/observations.jsonl {proc.forge.ready.address}
{experiment}/NOTICE`, under skein's `lab.md`, 7.2:

The helper is Python 3.11+ standard library only, under the same
"for now" as the runner (decision 253). It is lab tooling beside the
experiments in `temper/lab/`, never product code or a test in temper's
worlds. Its future application home is Rust. The lab's key records its
interpreter and executable identity (skein's `lab.md`, 2.3).

1. **The stream:** complete lines and a complete flush boundary with
   explicit loss count 0. Undecided, `observations missing`, when it
   cannot be read or completeness is unknown; `observations lost`,
   when a line counts any loss.
2. **Landed:** the last `Merged` of `fake/widget` into `main`, at commit
   C, and `main`'s last `Moved` to C. Otherwise it fails
   `not-landed/fake/widget/main`.
3. **The store:** C and its first parent P, by `objects` (7.2).
   Unreadable, it is undecided, `store unreadable`.
4. **The content:** C's `NOTICE` is exactly the experiment's `NOTICE`.
   Otherwise it fails `content/fake/widget/main/NOTICE`.
5. **One file:** P has no `NOTICE`, and C's tree is P's with `NOTICE`
   added and nothing else. Otherwise it fails `changed/fake/widget/main`.

- **It prints on stderr what it read:** C, P, both trees' paths and
  `NOTICE`'s bytes, kept as its evidence.
- **Independent:** it reads the forge's record and store, never temper's
  documents or the agent's report, and judges no promise. Its
  signatures name what failed and where, never a count or an address.

## 9. The planted fault

- **P's fault:** `{"kill": "serve", "signal": "SIGKILL",
  "after_actions": [1, 1]}`: `temper serve` alone killed once the
  driver's first action has ended (skein's `lab.md`, 4.6).
- **Its planted signature:** `stop/serve/signal:SIGKILL` (skein's
  `lab.md`, 7.1, as the runner spells it). A judged run of P lacking
  it fails `planted-missing/…`. This alone proves M1's stop detection.
  The story may have progressed before the first action returned;
  `check/end-state/not-landed/fake/widget/main` is an observed failure
  when present, never a required planted signature.
- **Nothing is discounted:** a survivor of serve's tree, should an
  agent's process run at the kill, is a failure and a finding of its
  own.
- **Detection:** P judged under `temper-m1` at H's subject, three seeds,
  verdict fail; its first failing seed run again; each planted
  signature filed, with its reproduction rate, and the planted stop classed
  `experiment` by the owner (skein's `lab.md`, 9.3).
- **Correction:** P′, P with `faults` and `planted` empty, is H's tree.
  Before P's first run the owner checks it: P's files are H's but
  `experiment.json`, and P's `experiment.json` with those two members
  emptied has H's bytes. For the planted stop finding classed
  `experiment`, the fix is H's digest; its recheck is H's key at that
  subject and class, judged pass, recorded as its `rechecked` (skein's
  `lab.md`, 9.4). Other failures need their own cause, class, fix and
  recheck (10.3); clearing the fault alone does not close them.
  A P that drifted from H is fixed
  in P, never paid for with three new seeds.
- **Detection stays:** P is kept, and a later run of it is an `occurred`
  on the closed findings.
- **One signature, several causes:** if P also produced a not-landed
  finding, a later not-landed in H matches it and reopens it; the owner
  classes it again from that
  run (skein's `lab.md`, 9.3).

## 10. The campaign

### 10.1 Cost classes

| Class | ready, command, stop | driver: seconds, actions, input tokens | triage: seconds, tokens |
|---|---|---|---|
| `temper-m1-calibration` | 60 s, 120 s, 10 s | 900 s, 80, 2,500,000 | 600 s, 1,000,000 |
| `temper-m1` | as calibration's | about 1.5 times the calibration run's measured values, rounded up | as calibration's |

- **The driver** is Codex `gpt-6.1-sol` at medium effort, in the lab's
  settings and the key (skein's `lab.md`, 2.3 and 6.1).
- **The director's rule, not new defaults.** `temper-m1`'s three values, the
  calibration's usage and its spend go to the director before any run
  under it.
- **Observed, not capped** (skein's `lab.md`, 6.5): a drive may pass
  `driver_input_tokens` by the turn that reported it, recorded; usage
  the host did not report is null, never 0, and totals count it apart.

### 10.2 Order and paid runs

| Step | Key | Paid driver runs | Then |
|---|---|---|---|
| 1. smoke | S, `temper-m1-calibration` | 0: three seeds, attended | pass, or its cause fixed and S judged again |
| 2. calibration | H, `temper-m1-calibration` | 1 | usage measured; if unusable, report before any replacement calibration |
| 3. class | | 0 | `temper-m1` set and reported |
| 4. healthy | H, `temper-m1` | 3, by `lab judge` | verdict pass |
| 5. planted | P, `temper-m1`, H's subject | 3, then 1 rerun | verdict fail, findings filed |
| 6. correction | H's key, as P′'s | 0 | findings closed with `--recheck` |

- **Eight of fifteen planned;** seven held for attempts after an error
  and one three-seed recheck of H at a subject carrying a product fix.
  Exactly one calibration is authorized; replacing it needs the
  director's decision. Past fifteen paid driver runs, the director is
  asked first.
- **Evidence is reused for an identical key alone** (skein's `lab.md`,
  2.3 and 8): H's verdict serves P′ because their experiment, subject
  and lab digests are the same, recorded in each finding. A change to H
  after step 4 is a new key, made only for a finding's fix.
- **Attempts:** a seed whose run ended in error or incomplete is run
  again once its cause is fixed, each a paid run.
- **The triager** runs only on a run with failures no signature
  matched, or with use failures (skein's `lab.md`, 9.2): P's first
  failing run, and H's should one arise; never a rerun that matched.

### 10.3 Findings

- **By class** (skein's `lab.md`, 9.3): `world`, `below` and `tools` are
  temper's; `fake`, `temper fake`'s (first-run's) or skein's fakes'.
  Each is fixed with a test in the lowest world that owns the behaviour,
  failing before the fix and passing after, before it lands
  (`domain/testing.md`; `client/tools.md`, 11.6), by that area's owner
  through `land`; its recheck is H at a subject that carries it.
  `experiment` is `lab/`'s, the QA owner's: its fix the corrected
  digest, its recheck that experiment's verdict. `lab` is skein's
  `scripts/lab/`.
- **Routed through the director** to the owner of what a finding names;
  the records stay under `~/src/rust/tmp/lab/`, and no issue is written.
- **A transcript that shows a temper bug** becomes a world's story: its
  calls are `temper ctl`'s (`client/tools.md`, 11.6).

### 10.4 The closing note

`campaigns/temper-m1.md`: the runner's tables (skein's `lab.md`, 10.2),
finished by the owner with each key's verdict; the findings filed and
closed, each with its reproduction rate; each run's usage and spend,
known and unknown, overshoots and triages; what the seeds fixed and
what they did not (skein's `lab.md`, 5.4); the choices M1 ran on (11.2);
and what the lab application should take over from the records and the
runner, and what to change in them.

## 11. What M1 waits for, and adopted decisions

### 11.1 Pending

The experiments are written to these as designed; each is its owner's,
and arrives through the director.

| What | Whose | For |
|---|---|---|
| the runner's driver, tool server, checks, judge and findings (skein's `lab.md`, section 10) | QA, in skein | every run |
| `temper fake llm` and `fake forge` with `--document`, `--ready`, `--seed` and `--observations`; `temper fake git` by its name; serve's three directories (`protocol/setup.md`, 8.4; `domain/testing.md`, 4.3; `protocol/configuration.md`, section 12) | first-run | the cluster |
| the fake LLM's lab form taking cued scripts with tool calls (7.1) | first-run | the story |
| the fake forge's lab form taking the domain's CI cue (7.2) | first-run | CI by content |
| `fake/widget`'s number, and its seed holding no `NOTICE`, its own checks, if any, passing with it | first-run | the delegate, the change |
| the observation writer and complete flush boundary before stop, with explicit cumulative loss count including 0; encoding is skein `527480f3` | first-run | 8.3, 8.4 |
| `temper fake forge check`, its Profile/Decision input design and temper-record decision source; judge landed at skein `4f13123c` | first-run's command and input design, github's review | 8.3; decision 259 |
| the git route's `objects` (skein's `fake-forge.md`, section 4) | skein's fake forge | 8.4 |
| TLS on the fake LLM with skein's test root, and the root's file to trust | first-run | M1; no plaintext variant |
| fake forge deliveries across processes to the hook registered on serve's bound webhook address | first-run | M1's story |
| serve's actual bound webhook address available to reconcile and a ready notification for its listeners | serve's CLI owner, routed from QA's S2 by the director | delivery readiness |
| fake-owned key/token derivation from decimal u64 seeds and 0600 output files named by the experiment; exact interfaces | first-run, director 253's default | seed, setup handover; runner invents no bytes |
| `wait forges --until connected:1` (area 6) | github | the seed, as the demo's |
| serve's link on its stdout, or a private channel for it | the director (the serve survey) | the seed's sign-in |

### 11.2 Decision 253

- **Deliveries:** M1 runs on webhooks. Polling remains only temper's
  own existing fallback. First-run supplies cross-process delivery;
  serve's owner supplies the bound webhook address and readiness.
  Reconcile registers the hook after serve binds.
- **Lab programs:** `tools/ctl` and `smoke/story` are POSIX sh.
  `checks/end-state` is Python 3.11+ standard library only, beside the
  experiments, under the runner's "for now"; never product code or a
  test in temper's worlds. No third-party modules, pip or virtualenv.
- **Observation completeness:** first-run supplies a flush boundary
  reachable before stop, with an explicit loss count, 0 included.
  Missing completeness or any loss is undecided. The lab does not
  infer zero from a missing loss record.
- **TLS:** H and S wait for fake LLM TLS with skein's test root. No
  plaintext variant.
- **Credentials:** the director's default, routed to first-run, is
  decimal u64 `--seed` alone; fakes derive their own key/token bytes
  and write needed outputs to 0600 files the experiment names. The
  runner invents no bytes. Derivation and output interfaces remain
  pending first-run's contract.
- **Content:** read the fake's store through the designed private git
  `objects` route, without an admin-token API workaround.
- **Cost:** 245's ready, command, stop and triage values stay. Only
  the driver's three measured bounds rescale by about 1.5 times the
  calibration's use, reported with the run's spend before use.
