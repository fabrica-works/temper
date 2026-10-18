# Configuration

Provisional, 2026-10-14, revised 2026-10-18. What temper's processes
are told when they start: the one JSON document `temper serve` reads,
and the forge's setup's commands with it, with jig's parts as jig
defines them and temper's own members beside them; the private files it
names; the agent's configuration; what `temper open` and `temper
sign-in` are given; limits; what holds at a first start and at every
start; and the formats an operator writes.
jig's shape and rules are jig's `protocol/configuration.md`; this
document says what temper adds. What is open is listed in section 11.

## 1. In one page

- **One document per `temper serve`,** in jig's shape: three top-level
  members, `engine` (jig's engine part), `workers` (jig's worker parts)
  and `application` (temper's members). Nothing else at the top level.
  The forge's setup's commands read the same document (section 8).
- **Read once, before the loop, and strict.** Unknown, missing,
  duplicate or out-of-bound members refuse the start, naming the path
  (jig's `protocol/configuration.md`, section 1). A change is a restart.
- **jig's parts are jig's:** `jig-protocol-config` parses them; temper
  writes no parser of its own for them, and adds only the checks
  standalone needs (section 2).
- **temper's members,** under `application`: `forges`, `identities`,
  `landing`, `charter`, `clients`, `webhooks` and `limits` for the
  engine (section 3), and `workers`, one entry per worker (section 4).
  `webhooks` is the forge component's webhooks' listener, on loopback
  (forge.md, section 5).
- **`setup`, for the forge's setup** (setup.md): how each set-up forge is
  run and set up, which the `temper forge` commands and `temper box` act
  on, and `temper serve` checks and acts on none of (section 3).
- **Not configuration:** repositories, which people adopt at runtime;
  the branch namespace, which the root derives from the deployment's id;
  the forge writer's identity, which comes from each forge's `user`.
- **No secret in the document.** Each secret is a private file the
  document names, read at start (section 6).
- **One format rule** for temper's typed members: member names are the
  Rust field names, and the parser follows the types (section 10).
- **Standalone shape:** exactly one worker, joined to the engine by the
  in-process name `workers`; the clients' listener, and the webhooks'
  listener when configured, on loopback; the agent program is temper's
  own binary.
- **The agent's configuration is smith's** (section 5), and `temper open`
  takes no file at all (section 9).

## 2. The document

```
{
  "engine":      jig's engine part
  "workers":     [ jig's worker part ]
  "application": temper's members
}
```

- **`engine`** is jig's engine part (jig's `protocol/configuration.md`,
  section 3), in jig's formats (its section 10). Its `clients` member is
  what jig's clients' component takes; the clients' listener is temper's
  (section 3).
- **`workers`** is a list of jig's worker parts (jig's
  `protocol/configuration.md`, section 4).
- **`application`** holds temper's members (sections 3 and 4).
- **jig's reference fixtures** are a complete example of jig's parts: the
  reference binary's test configuration (`jig-test`'s
  `tests/support/mod.rs`, made from jig's channel world's `fixtures.rs`).
  temper's process tier writes its own the same way (`../testing.md`,
  2.6), and the owned example is a whole document (setup.md, 8.1).
- **What standalone requires of jig's parts,** each checked at start, a
  failure naming its path:
  - `workers` has exactly one entry, whose `engine` is `{"in_process":
    "workers"}`;
  - `engine.hosts.listener` is `[{"in_process": "workers"}]`, and
    `engine.hosts.workers` names that one worker;
  - `engine.connectors` is exactly `[1]`: the forge connector, temper's
    one connector, whose number both ends of the client's wire fix as
    `temper-client-protocol-wire`'s `FORGE` (`../client/README.md`, 4.6);
  - the worker's `agent` paths are absolute (section 5).

  The worker's `agent` names temper's own binary (section 5), which no
  check can tell from another program.
- **What temper's root requires of jig's parts,** standalone or not, each
  checked at start, a failure naming its path: `engine.limits`'
  `fleet.engine_slots` is 0, and every charter's `hosts` is `"workers"`.
  The root has no hub child, so the fleet must place every run on a
  worker (`../domain/root.md`, section 2).
- **Templates are jig's two defaults,** chat (template 1, label 1) and
  goal (template 2, label 2), as jig's `protocol/configuration.md`,
  3.1 gives them, the chat's executor the charter chats run under;
  temper defines no template of its own. Its client's lists filter on
  those labels (`../client/README.md`, 3.5), and a project's policy may override
  their defaults as jig's allows.
- **A project's permission mapping,** jig's `roles` of each of
  `engine.projects` (jig's `domain/people.md`, section 4), names the
  forge's permissions by number under the forge connector, 1: 1 read, 2
  write, 3 admin. An adoption gives a repository's collaborators the
  roles their permissions map to (`../domain/forge.md`, section 4).
- **Reading,** by `temper serve`, in this order:
  1. the file, whole, with skein's startup kit, within the bound jig's
     reference binary sets on a configuration's bytes;
  2. jig's parts, by `jig-protocol-config`; `application`, by
     `temper-service`'s startup, with `jig-protocol-config`'s readers, so
     every problem names its path the same way
     (`application.forges[0].user`); `setup`, and the setup's and the
     box's limits, by their own crates' checks (setup.md, sections 8 and
     11);
  3. the checks across the whole: the references of sections 2 to 5,
     the limits against each other and the worst case (section 7), and
     the channel's start checks on temper's parts (channel.md, section
     8);
  4. every private file the document names, each within its bound
     (section 6), then each identity's whole value against the channel's
     bound on a grant's value (section 3);
  5. the agent's configuration (section 5).

  Every problem found is reported, one line each on stderr, and `temper
  serve` exits 1 before io starts.
- **Paths** are resolved against the process's working directory, as
  jig's reference binary resolves them, but the agent's, which are
  absolute (section 5).

## 3. temper's members for the engine

**`forges`:** a list of the forges temper works on, at least one, and at
most the forge client's `repositories` limit (`forge.client.repositories`),
which bounds its writers. Each:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `number` | u16 | unique among `forges` | the domain's forge number: a repository's `forge`, a writer's, a git identity's |
| `kind` | string | `"forgejo"`, the only kind | the documents the forge component speaks (forge.md, section 3) |
| `api` | an address, jig's format | a socket address, or a host's name and port; never an in-process name | where the forge component connects for the forge's REST API, under `/api/v1` |
| `transport` | `"plaintext"`, or `{"tls": {"server_name": string, "trust": file or null}}` | plaintext only when `api` is a socket address on loopback; `trust` null is the machine's roots | the forge component's connections |
| `git` | string | an `http` or `https` URL: a host, an optional port and an optional path; no user, query, fragment or final `/`; `http` only to a loopback host | the base of each repository's remote, `<git>/<owner>/<name>.git`, which the engine's routing makes (channel.md, section 4); and the host segment of the forge's resource paths: `git`'s host, with `:port` when it names a port |
| `api_token` | a file | a private file (section 6) | the forge component: the API token (credentials.md, section 5) |
| `user` | u64, or `{"file": path}` | above 0; a file is a private file (section 6) holding the id in decimal (section 10), read at start | the forge user the API token belongs to: the forge writer's identity, checked at start (credentials.md, section 5) |
| `identity` | u32 | the `number` of one entry of `identities`; each identity named by exactly one forge | the git identity this forge's runs use (credentials.md, section 4) |
| `webhook_secret` | a file, or `null` | a private file (section 6); a file while `webhooks` is `null` refuses the start, naming its path | the forge component's webhooks' listener: the secret each delivery to this forge is signed with (forge.md, section 5; credentials.md, section 2). A forge with none has no path on the listener |

**`identities`:** a list of git identities, temper's application
credentials (credentials.md, section 4), as many as `forges`, and at most
jig's limit on an engine's application credentials (the accounts child's
`credentials`). Each:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `number` | u32 | unique among `identities`; from 1 to the accounts child's `credentials`, in `engine.limits` and in the worker's `limits`, since the worker's identities' table holds those numbers alone | the credential's number, as the root names it in `Event::Credential` and in grants |
| `token` | a file | a private file (section 6) | the token git authenticates with |
| `login` | string | not empty; no `:` and no control byte | the user of git's HTTP basic authentication |
| `author` | `{"name": string, "email": string}` | each not empty; no NUL byte, newline, `<` or `>` | the author and committer of the commits made with it (git.md, section 5) |

The whole value, token, login and author encoded (channel.md, section
6), must fit the channel's bound on a credential's value, or the start is
refused, naming the identity.

**`landing`:** the deployment's and the projects' landing rules, the
root's `LandingPolicy`:

```
"landing": {
  "deployment": [ rule ],
  "projects": [ {"project": u32, "rules": [ rule ]} ]
}
```

A rule is a `LandingRule`, member for member:

| Member | Type | Meaning, and bounds |
|---|---|---|
| `connector` | u16 | the forge connector's number, the one `engine.connectors` names |
| `kind` | u16 | the effect kind the rule governs: 4, the forge's merge, the only kind a landing rule takes |
| `pattern` | jig's pattern, as jig's `rules` and `projects` write one: `{"segments": [string], "last": "exact" or "open", "value": string}` | the resources the rule covers, as jig's authority matches them: the exact base segments, then one more segment equal to `value` (`exact`), or beginning with it, with all below it (`open`). The forge's resource path is `forge`, the forge's host (as `git` gives it), the repository's owner and name, then `branch` and the branch's segments |
| `enforce` | boolean | true makes each criterion below a requirement every landing it covers must meet (jig's `domain/authority.md`, section 10; `../domain/forge.md`, 8.7); false leaves them to the change procedure alone |
| `ci` | boolean | CI must have passed at the change's head |
| `up_to_date` | boolean | the change's head must contain its base's tip |
| `gates` | list of `{"number": u32, "blocking": boolean, "freshness": "exact" or "clean"}` | the gates every change it covers asks for (`../domain/forge.md`, 8.3): an agent's review, or one of a repository's checks when it has no CI; at most the forge's `change_policy.gates` |
| `approvals` | list of `{"role": u32, "people": u32, "freshness": "exact" or "clean"}` | `people` distinct people holding `role` in the project approve the change, each by a review on the forge (`../domain/forge.md`, 8.7) |

- **`freshness`,** a gate's or an approval's, says at which later heads
  a verdict still counts: `exact`, the head it was given at; `clean`,
  every head reached from it by updates temper made that merged cleanly
  (`../domain/forge.md`, 8.3). It is not jig's freshness of an observed
  requirement, the age within which its facts must have been read: CI,
  gates and approvals are guarded, and up to date is observed with a
  freshness of 0 (`../domain/forge.md`, 8.7).
- **Bounds:** each list of rules holds at most jig's authority limit on
  requirements; a project appears in `projects` at most once, and must
  be one of `engine.projects`; each approval's `role` must be a role of
  its project's policy, and, for a deployment rule, of every configured
  project's policy.
- **Feeds** `Config.landing`: `deployment` and `projects` as they are.

**`charter`:** u32, the charter the forge's procedures start agent work
under; it must be the `number` of one of `engine.charters`. Feeds
`Config.forge_charter`.

**`clients`:** the clients' listener, which `temper-service` owns
(`../client/README.md`, section 8):

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `listener` | an address, jig's format | a socket address; on loopback (in `127.0.0.0/8`, or `::1`) in standalone, and wherever `engine.clients` enables the printed link (jig's `protocol/service.md`, section 3) | the listener on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig`; the mapping's Host allow-list; and the address of the sign-in links `temper serve` prints (`../client/README.md`, section 7) |

**`limits`:** temper's profile and overrides (section 7).

**`webhooks`:** the webhooks' listener, the forge component's own
(forge.md, section 5), or `null` for none:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `listener` | an address, jig's format | a socket address on loopback (in `127.0.0.0/8`, or `::1`), as the clients' listener's in standalone; unique among the deployment's listeners: neither the clients' listener's nor an owned forge's `listen` (setup.md, section 8) | the forge component's listener for Forgejo's deliveries, plaintext HTTP on skein's HTTP server, one path per forge with a `webhook_secret`, `/hooks/<number>` by the forge's `number`; the URL of each org hook reconcile makes (setup.md, section 5); an owned Forgejo's `[webhook] ALLOWED_HOST_LIST` (setup.md, 6.2) |

- **A delivery is a hint,** never a fact: polling is the same with
  webhooks or without (forge.md, section 6), and finds what a delivery
  lost, refused or never sent told of.
- **A forge on another host polls:** its deliveries would need TLS on
  the listener (README.md, section 14; setup.md, section 15).

**`setup`:** the forges' setup, a list of at most one entry per forge,
`[]` for none, and never missing (section 10). Its members, their bounds
and the checks of the forge's own members against each entry are
setup.md, section 8's. `temper serve` checks it whole, and acts on none
of it: it reads no file that only `setup` names.

**What the engine root's `Config` takes,** field by field:

| Field | From |
|---|---|
| `core` | `engine`, by `jig-protocol-config`, with the deployment's id and seed |
| `landing` | `application.landing` |
| `forge.writers` | one `Writer { forge: number, author: user }` per entry of `forges` |
| `forge.namespace` | not configured: the root derives it from the deployment's id |
| `identities` | one `GitIdentity { forge: number, credential: identity }` per entry of `forges` |
| `forge_charter` | `application.charter` |

Beside the root: the forge component takes each forge's `number`,
`kind`, `api`, `transport`, `api_token`, `user` and `webhook_secret`,
and the webhooks' listener, `webhooks.listener`; the identities' table
takes each identity's value; the engine's routing takes each forge's
`git`; the clients' listener takes `clients.listener`.

## 4. temper's members for a worker

**`application.workers`:** a list with one entry per entry of the
top-level `workers`, in the same order; standalone has one. Each:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `root` | a directory | made private (0o700) if absent; an existing one must be the process's user's, private, and not reached through a link | the worker's workspace directory: every checkout's workspace is made under it (git.md, section 2; `../domain/worker.md`, section 5) |
| `git` | string | an absolute path to an executable file, checked at start, whose `git version` is at least the one git.md, section 3 names | the program the git component runs for each invocation, in the environment git.md, section 5 gives it |

- **The worker's root takes the rest from jig's and smith's:** the hub's
  limits from jig's worker part; smith's host limits and the bounds its
  hello states from the agent's configuration (section 5), as jig's
  reference binary derives them; its stop bound from jig's graces; the
  checkout's limits from temper's profile (section 7).
- **Workers are alike** (jig's `protocol/configuration.md`, section 4):
  no credential but the worker's secret, and no forge configuration.
  Remotes and git identities come with each assignment.

## 5. The agent's configuration

- **smith's agent configuration** (smith's `protocol/agent.md`, section
  4), a file of smith's, named by jig's worker part's `agent` member:
  - `program`: the absolute path of temper's binary;
  - `configuration`: the absolute path of smith's agent configuration;
  - `arguments`: `["agent", <that same path>]`,

  so each agent process is `temper agent AGENT.json` (README.md,
  section 4).
- **Read twice:** by `temper serve` at start, for smith's host limits,
  the channel's limits and the agent's bounds, as jig's reference binary
  reads it; and by each agent process as it starts.
- **It holds no credential:** an agent's grants come over smith's
  channel (credentials.md, section 3).
- **The same names as the engine's** (jig's `protocol/configuration.md`,
  section 4), checked at start: every endpoint of `engine.llm.endpoints`
  must be in the agent's configuration under the same name with the same
  account, and every model a charter names must be declared there, or
  the start is refused, naming it. One process reads both, so a mismatch
  that would fail every run's charter is caught before the loop.

## 6. Secrets' files

| File | Named by | Holds | Read by |
|---|---|---|---|
| a forge's API token | `application.forges[].api_token` | the token's bytes | the forge component |
| a forge's user's file | `application.forges[].user`'s `file` | the user's id, in decimal | `temper serve`'s startup, for the forge component |
| a forge's webhooks' secret | `application.forges[].webhook_secret` | the secret's bytes | the forge component's webhooks' listener |
| a git identity's token | `application.identities[].token` | the token's bytes | the identities' table |
| a managed forge's admin's token | `application.setup[].mode.managed.admin_token` | the token's bytes | `temper forge reconcile` alone (setup.md, section 5) |
| a worker's secret, its digest, a handed-in LLM account's file, the accounts' records' directory, an LLM account's client secret | jig's parts | jig's formats (jig's `protocol/configuration.md`, sections 7 and 10) | jig's components |

- **A secret file holds its bytes and nothing else** but a final
  newline, which is not part of the secret (jig's
  `protocol/configuration.md`, section 10).
- **Read at start,** each within its bound, with skein's
  `read_private_file` (skein's `shell.md`, 6.1): a file readable by
  anyone but the process's user, or reached through a link, refuses the
  start, naming it. An API token's bound is the forge component's
  `token_bytes` (forge.md, section 7), and so is an admin's token's; a
  webhooks' secret's is the forge component's `hook_secret_bytes`
  (forge.md, section 7); a user's file's is 20 digits; an identity's
  token's is the channel's bound on a grant's value, which its whole
  value, login and author included, must fit (section 3; channel.md,
  section 8).
- **A set-up forge's directory** (`application.setup[].directory`) and
  its files are setup.md, section 3's; the forge's `api_token`, `user`
  and `webhook_secret`, and its identity's `token`, name them (setup.md,
  section 8).
- **Never written by `temper serve`.** The setup's commands write the
  files of a set-up forge's directory (setup.md, section 3); an operator
  replaces any other and restarts (credentials.md, sections 4 and 5).
- **No temper secret in jig's secrets' store** (credentials.md, section
  2).

## 7. Limits

```
"limits": {"profile": "small", "overrides": {"<name>": value}}
```

- **Every limit temper's own domains and components take:** the forge's
  and its change and issue policies, the queue window, the forge
  client's, the checkout's, the root's own beyond jig's core, the forge
  component's, its webhooks' listener's among them, and the git
  component's; and the setup's and the box's, which `temper serve`
  checks and takes none of (setup.md, sections 5 and 7).
- **One profile, `small`,** for one person's machine; another name is
  refused. Its numbers are constants in `temper-service`'s limits module,
  and the setup's and the box's in their own crates, each documented
  with what it bounds; the design names limits and what they bound, and
  never their numbers (jig's `protocol/configuration.md`, section 6).
  Where a limit of temper's is a quantity jig's profile also sets, its
  constant cites jig's small profile's.
- **`small` starts as it stands,** with no override, against the agent
  temper runs: smith's, whose run bounds (its directories, their names,
  its accounts) are smith's own and no configuration changes (smith's
  `smith-domain-run`, `derive`; `smith-agent-service`), and whose largest
  call, its run's `outcome_bytes`, follows from its agent configuration,
  the owned example's (setup.md, 8.1). Every constant passes the
  checks below with that agent: a task's resources and the checkout's
  repositories within the agent's directories, the checkout's names
  within theirs, and a run's push message, which smith bounds as a call,
  within the checkout's commit message. So `small` takes a forge's `git`
  base of at most 122 bytes, the longest remote's bound (below). A test
  whose agent has other bounds states the overrides it needs.
- **Overrides by dotted name:** the path of the field in its owner's
  `Limits` type, a duration's last segment with `_ms`. A name temper does
  not know, or a value outside its type, refuses the start, naming it.

  | Prefix | Owner |
  |---|---|
  | `root.` | the engine root's own limits, beyond those jig's `engine.limits` sets |
  | `forge.` | `temper-engine-domain-forge`'s `Limits`: `forge.client.`, `forge.change_policy.` and `forge.issue_policy.` for its children, `forge.queue_window_ms` |
  | `checkout.` | `temper-worker-domain-checkout`'s `Limits` |
  | `forge_component.` | the forge component's, its webhooks' listener's among them (forge.md, section 7) |
  | `git_component.` | the git component's (git.md, section 8) |
  | `setup.` | `temper-forge-setup`'s `Limits`: reconcile's (setup.md, section 5) |
  | `box.` | `temper-box`'s `Limits`: the box's restarts and stop (setup.md, 7.2) |

  For example `forge.repositories`, `forge.client.calls`,
  `checkout.remote_timeout_ms`.
- **The webhooks', the keep's settling, the setup's and the box's
  limits,** whose numbers are `small`'s constants, as every other's:

  | Name | Bounds |
  |---|---|
  | `forge.client.settle_ms` | after a move of a pull request's head or base, how long a not mergeable answer, or a hinted job's pending CI, is read again at `hinted` and not believed, by the change procedure (`../domain/forge.md`, 5.2); above 0, as `hinted` is |
  | `forge_component.hooks` | the webhooks' listener's connections in all |
  | `forge_component.hook_head_bytes` | a delivery's request head |
  | `forge_component.hook_body_bytes` | a delivery's body, hashed and decoded as it streams, so time and bytes, not memory; one past it is answered `413` and hints nothing |
  | `forge_component.hook_deadline_ms` | `hook_deadline`: a delivery's read and answer, from its connection's accept or its last answer |
  | `forge_component.hook_secret_bytes` | a webhooks' secret |
  | `setup.orgs` | a setup's `orgs` |
  | `setup.deadline_ms` | a reconcile's run, its wait for the forge among it |
  | `setup.backoff_ms`, `setup.backoff_max_ms` | reconcile's first request made again, doubling |
  | `box.backoff_ms`, `box.backoff_max_ms` | a long step restarted, doubling |
  | `box.restarts`, `box.window_ms` | a long step's exits within the window past which the box gives up; `box.restarts` at most 64; the backoff counted from the exits within the window (setup.md, 7.2) |
  | `box.grace_ms` | a long step's grace, after each of its stop's two signals (setup.md, 7.2) |
- **Limits checked against each other at start,** each failure naming
  both limits:
  - the forge component's: its connections equal the forge client's
    calls in flight; its deadline is at most the forge client's
    lifetime; its body bound holds the forge client's largest write and
    a key's marker; every listing's page size is at least 1 (forge.md,
    section 7);
  - the git component's: each rule of git.md, section 8 (its operations
    at least the checkout's workspaces, its output caps at least what
    it parses and the diagnostic it keeps);
  - the setup's and the box's: each backoff at most its maximum;
    `setup.backoff_max_ms` below `setup.deadline_ms`; `box.backoff_max_ms`
    below `box.window_ms`;
  - the webhooks' listener's: each of its limits above 0;
    `hook_secret_bytes` at least 32; `hook_head_bytes` at most the
    HTTP server's largest read within io's intake; `hooks` within the
    sockets left once the clients' listener and the forge's connections
    are counted. The sockets are temper's own, a constant of
    `temper-service`'s limits module that its io check takes in place
    of the sum of jig's engine's and worker's, and `small`'s hold a
    document with both listeners, as the owned example has (setup.md,
    8.1);
  - the channel's (channel.md, section 8): each of temper's largest
    parts within the ceiling jig's schema gives its field; the largest
    part the engine can send within what the worker's limits take; the
    longest remote the routing can make (the longest `git` base, an
    owner and a name of the forge's `name_bytes`, two `/` and `.git`),
    and the forge's names and branches, within the checkout's
    `name_bytes`;
  - the worker's, against the agent's configuration
    (`temper-worker-domain`'s `worst_case`), among its rules: the
    checkout's repositories, names, conflicts and paths within the
    agent's directories, their names, conflicts and paths; the agent's
    largest call within the checkout's commit message; and, of the hub's
    (jig's, in the worker's `limits`), its accounts within the agent's,
    its largest event with smith's 2-byte separator within the agent's
    message, and the agent's message less that separator within the
    hub's event. So `hub.event_bytes` is the agent's `message_bytes` less
    2, and no other value starts.
- **jig's limits stay in jig's parts:** `engine.limits` and each worker's
  `limits` (jig's `protocol/configuration.md`, section 6).
- **What the root takes of `engine.limits`:** its core children's
  (`tasks.`, `authority.`, `people.`, `fleet.`, `accounts.`, `notes.`
  and `documents.`), with `core.templates` and `core.call_records`. The
  rest of the core's it gives the core itself (`../domain/root.md`,
  section 10): from its own `root.` limits, in place of `brief.`,
  `core.brief_parts`, `core.brief_core_budgets.`, `core.resume_bytes`,
  `core.run_bytes`, `core.policy_bytes`,
  `core.escalation_reason_bytes`, `core.call_answer_bytes` and
  `core.load_slots`; as constants, in place of `views.`,
  `core.fill_mark`, `core.load_bytes` and `core.connectors`. An
  override of one of these changes nothing the root holds.
- **The engine's documents fit temper's client.** A document past the
  bounds a client decodes is refused there whole, as too large (jig's
  `protocol/client.md`, section 3), so the documents the engine serves
  must be within what temper's client takes: its client takes
  `jig-client`'s limits from jig's small profile with the same
  `documents.` overrides the owned example's `engine.limits` names
  (`../client/README.md`, section 2; setup.md, 8.1). The engine cuts each
  text to its bound and says so (jig's `client/domain/contract.md`,
  5.3). It checks none of this at start: it serves whatever client the
  deployment lets in, whose limits are the client's. A test holds the
  owned example's and the process tier's configuration to temper's
  client's limits, so the two cannot drift apart unseen.
- **The engine's watch room fits temper's client.** A watch past the
  room the engine allows a party, or a stream, is refused busy, and each
  holds its place until the engine has ended it, so a deployment allows
  each party, and each stream, at least the watches one client opens at
  its limit, `me`'s included (jig's `client/domain/contract.md`, 7.2;
  jig's `protocol/client.md`, section 13). temper's client opens 16, so
  its engine allows 16 a party, 16 a stream and 16 in all, temper's own
  numbers above jig's profiles' 4, which the root holds whatever
  `engine.limits` says (`../domain/root.md`, section 10). It checks none
  of this at start, as for documents (above); the same test holds these
  rooms to temper's client's watches.
- **The forge's paths fit a document.** jig's documents carry a
  resource's path and a pattern whole, with as many segments as the
  engine admits in a name (jig's `client/domain/contract.md`, 5.3), and
  `documents.lines` bounds a document's short lists alone, at most the
  client's rows. So `engine.limits` must name the bounds on a name's
  segments, `authority.segments`, `tasks.hold_segments` and
  `tasks.authority_segments`, at least the segments of the deepest path
  the forge names: a branch's, the forge's five segments then the
  branch's own (`../domain/forge.md`, section 2), 8 as temper's branches
  name them. temper's client takes a path's segments, 9, apart from its
  rows (jig's `client/domain/client.md`, section 14), so a page of the
  forge's work is never refused for a path, and a long path shows cut,
  with jig's mark (`../client/README.md`, 5.4).
- **The forge's authorities fit a document.** jig's documents carry an
  authority's lists whole, with as many entries as the engine admits in
  each, and neither cut nor refuse a document or an offer for one (jig's
  `client/domain/contract.md`, 5.3), so a party who may amend a task
  reads its document whatever `documents.lines` is. temper's client
  takes an authority's entries, 12, apart from its rows (jig's
  `client/domain/client.md`, section 14): the most the process tier and
  the owned example admit in a list, their `authority.grants` and
  `tasks.authority_grants` (setup.md, 8.1). A long authority shows
  in part, with jig's mark (`../client/README.md`, 5.4). The test above holds
  the engine's bounds on an authority's lists to temper's client's.
- **One worst case per service:** jig's sum (the root's, its components'
  and the journal's) and temper's (its domains', its components', the
  identities' table) together must fit the memory jig's part states,
  `engine.memory_bytes` for the engine's service and `memory_bytes` of
  the worker's part for the worker's, or the start is refused, saying by
  how much. The owned example's `engine.limits`, worker `limits` and
  `memory_bytes`, and why each holds, are setup.md, 8.1's.

## 8. First start and every start

- **Every command that reads the document checks it.** `temper serve`
  checks it whole, as section 2 says. `temper sign-in`, every `temper
  forge` command and `temper box` check its structure, its members, the
  limits and `setup`, and none of the checks that need a private file
  read, since a set-up forge's files do not exist before reconcile makes
  them (setup.md, section 8). Each reads the private files its own part
  needs alone: a `temper forge` command its forge's, by setup.md,
  sections 5 and 6; `temper box` none; `temper serve` every one the
  document names but those that only `setup` names. A set-up forge's
  files that its members name, which the commands before it made, are
  among the ones it reads.
- **A deployment's first start** is jig's (jig's
  `protocol/configuration.md`, section 8): the first with an empty store,
  whose owners and projects the core seeds from `engine`. temper adds:
  - **periods from the smallest source,** as jig's reference service
    gives them (jig's `protocol/configuration.md`, section 5): before the
    core admits a party's request, the engine's routing gives the root
    one period for each configured project, its budget the project's
    spend per period, with no next start;
  - **each project's landing rules** join its policy as it is seeded.
- **Every start:**
  - jig's parts apply as jig says (jig's `protocol/configuration.md`,
    section 8);
  - temper's members apply as configured: the forges, the identities,
    the deployment's landing rules, the forge's judges for every rule,
    the charter, the limits and the workers;
  - every limit is checked again, temper's and jig's together, with the
    channel's start checks on temper's parts (section 7; channel.md,
    section 8), so a `git` base or a limit changed since the last start
    is refused before io starts when a remote or a part could pass its
    bound;
  - the root's configuration is held to the root's limits before the
    root is made (`../domain/root.md`, section 6): a start that breaks
    one is refused, naming it;
  - every identity is announced at generation 1 before the restart
    (credentials.md, section 4), and every forge's user is checked by the
    forge component's first exchange (credentials.md, section 5);
  - the branch namespace is derived from the deployment's id, the same
    at every start;
  - a later start gives no period: the store keeps each project's
    current one (jig's `domain/engine.md`, 5.4).
- **Repositories are never configured.** A person adopts each at runtime
  through the client, by the forge's number, the owner and the name, and
  the store keeps it (jig's `protocol/configuration.md`, section 3;
  `../client/README.md`, section 4).
- **What a change reaches,** after its restart: a forge's `git`, every
  remote from the next assignment on; a forge's `api_token` and `user`,
  checked again; a forge's `webhook_secret`, and the webhooks' listener,
  from the first delivery on; an identity's token, granted again at
  generation 1; a forge's setup, at the next run of its commands
  (setup.md, 6.3 and section 5).

## 9. The client's arguments

- **`temper open DIRECTORY LINK_OR_ADDRESS`** takes no configuration
  file: its two arguments are `../client/README.md`, 6.1's, and its limits are
  constants of temper's client (`../client/README.md`, section 2).
- **`temper ctl DIRECTORY ...`** takes none either (`../tools.md`,
  section 8).
- **`temper sign-in CONFIG.json ACCOUNT`** reads the same document as
  `temper serve` (credentials.md, section 3).
- **`temper agent AGENT.json`** reads smith's agent configuration
  (section 5).
- **`temper forge prepare|run|reconcile CONFIG.json FORGE`** and
  **`temper box CONFIG.json`** read the same document as `temper serve`
  (section 8; setup.md, sections 5 to 7).

## 10. Formats

**The format rule** for temper's typed members, from the Rust types they
feed:

- **member names** are the field names;
- **an enum's variant without fields** is a string of its name in
  snake_case (`"clean"`);
- **a variant with fields** is an object with one member named after the
  variant in snake_case, whose value is the field's for one unnamed field,
  or an object of the fields by name (a forge's `transport`:
  `{"tls": {"server_name": ..., "trust": ...}}`);
- **byte strings** are JSON strings, UTF-8;
- **durations** are integers of milliseconds, the member's name suffixed
  `_ms`;
- **an absent option** is `null`, never a missing member;
- **a map keyed by a number** is a list of objects, each holding its key
  under the key's name (`landing.projects`: `project`);
- **integers** are within their type's range; **booleans** are `true` or
  `false`;
- **a value of one of jig's types** is written as jig's configuration
  writes it, so one document spells one type one way: a landing rule's
  `pattern`, jig's authority pattern, is
  `{"segments": [...], "last": "exact" or "open", "value": ...}`, as in
  jig's `rules` and `projects`.

**jig's formats,** by citation (jig's `protocol/configuration.md`, section
10): an address (`{"socket": ...}`, `{"host": ..., "port": ...}`,
`{"in_process": ...}`); a secret file; a digest file; a handed-in
account's file. A forge's `transport` offers the choices jig's LLM
endpoints and accounts offer (plaintext to a loopback socket address
only; TLS with a server name and a trust file, or the machine's roots),
written by the format rule as one member of temper's own type, where
jig's endpoints spell them as sibling members (`"transport": "tls"`,
`"server_name"`, `"trust"`).

**A user's file,** temper's own, holds the forge user's id in decimal
digits, and nothing else but a final newline, as a secret file does.

**The `application` member for standalone,** one forge, one identity and
one worker, complete:

```json
"application": {
  "forges": [{
    "number": 1,
    "kind": "forgejo",
    "api": {"host": "forge.example", "port": 443},
    "transport": {"tls": {"server_name": "forge.example", "trust": null}},
    "git": "https://forge.example",
    "api_token": "secrets/forge-1.api-token",
    "user": 42,
    "identity": 1,
    "webhook_secret": null
  }],
  "identities": [{
    "number": 1,
    "token": "secrets/forge-1.git-token",
    "login": "temper-bot",
    "author": {"name": "temper", "email": "temper-bot@forge.example"}
  }],
  "landing": {
    "deployment": [],
    "projects": [{
      "project": 1,
      "rules": [{
        "connector": 1,
        "kind": 4,
        "pattern": {
          "segments": ["forge", "forge.example", "acme", "widget", "branch"],
          "last": "exact",
          "value": "main"
        },
        "enforce": true,
        "ci": true,
        "up_to_date": true,
        "gates": [{"number": 1, "blocking": true, "freshness": "clean"}],
        "approvals": []
      }]
    }]
  },
  "charter": 2,
  "clients": {"listener": {"socket": "127.0.0.1:7100"}},
  "webhooks": null,
  "setup": [],
  "limits": {"profile": "small", "overrides": {}},
  "workers": [{"root": "workspaces", "git": "/usr/bin/git"}]
}
```

A landing rule may name a repository no project has adopted yet: it
covers nothing until a person adopts `acme/widget`. The forge is reached
over TLS from another host, so it polls, with no webhooks.

**The whole document,** jig's members elided as `...` (this skeleton is
not itself valid JSON):

```
{
  "engine": {
    "deployment": ..., "owners": ..., "providers": ..., "rules": ...,
    "projects": [{"number": 1, ...}],
    "charters": [{"number": 1, ...}, {"number": 2, ...}],
    "templates": [{"number": 1, ...}, {"number": 2, ...}],
    "hosts": {
      "workers": [{"name": "worker", "digest": "secrets/worker.digest"}],
      "listener": [{"in_process": "workers"}]
    },
    "llm": ...,
    "clients": {"link": true, ...},
    "connectors": [1],
    "limits": {"profile": "small", "overrides": {"authority.segments": 8, ...}},
    "memory_bytes": ...
  },
  "workers": [{
    "engine": {"in_process": "workers"},
    "name": "worker",
    "secret": "secrets/worker.secret",
    "slots": ..., "graces": ...,
    "agent": {
      "program": "/usr/local/bin/temper",
      "configuration": "/home/me/temper/agent.json",
      "arguments": ["agent", "/home/me/temper/agent.json"]
    },
    "limits": {"profile": "small", "overrides": {"hub.event_bytes": ..., ...}},
    "memory_bytes": ...
  }],
  "application": { ... as above ... }
}
```

Its `engine.limits` names the overrides section 7 asks, and its worker's
`limits` those the worker's check against the agent asks (section 7).
The owned example, `docs/examples/owned.json`, is a whole document for
one owned forge, with its own values (setup.md, 8.1).

## 11. Open questions

- **A calendar for periods.** The smallest source opens one period per
  configured project at the first start and none after, so a project's
  budget is not renewed. Default: the smallest source; a calendar child
  of temper's root, as `ops`'s (jig's `protocol/configuration.md`,
  section 5; jig's `examples.md`, section 4), would renew it.
- **A forge removed while the store holds its repositories.** Default:
  the start is refused at the forge's restart step, naming the forge
  number (jig's `protocol/configuration.md`, section 9, removed parts).
- **A project's landing rules changed after its first start.** They
  reach the forge's judges at the next start, but the project's policy
  keeps the requirements seeded at its first start. Default: a change
  reaches the policy only through a policy change by a person.
- **A repository renamed or moved after adoption.** Default: its
  resource path, which landing patterns match, and its remote keep the
  owner and name of its adoption; Forgejo redirects the old path, and
  git follows the redirect (`tools/forgejo-conformance/facts.txt`,
  `moved-repository-is-a-redirect`; `tools/git-transcripts/facts.txt`,
  `git-follows-a-renamed-repository-redirect`).
- **Profiles' numbers,** from the first deployments' measurements
  (jig's `protocol/configuration.md`, section 9). Default: `small` as
  written.
- **`small`'s worst cases against a person's machine.** With the owned
  example's strings at their bounds, the engine's and the worker's worst
  cases are past any machine's memory (setup.md, 8.1 and section 16), so
  the start's check (section 7, one worst case per service) bounds
  nothing a person's machine holds. Which terms make them so large is
  not known. Default: the check as it stands.
- **A configuration's format version** (jig's `protocol/configuration.md`,
  section 9). Default: none; unknown members refuse the start.
