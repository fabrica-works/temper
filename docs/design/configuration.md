# Configuration

Provisional, 2026-10-14, revised 2026-10-16. What temper's processes
are told when they start: the one JSON document `temper serve` reads,
with jig's parts as jig defines them and temper's own members beside
them; the private files it names; the agent's configuration; what
`temper open` and `temper sign-in` are given; limits; what holds at a
first start and at every start; and the formats an operator writes.
jig's shape and rules are jig's `protocol/configuration.md`; this
document says what temper adds. What is open is listed in section 11.

## 1. In one page

- **One document per `temper serve`,** in jig's shape: three top-level
  members, `engine` (jig's engine part), `workers` (jig's worker parts)
  and `application` (temper's members). Nothing else at the top level.
- **Read once, before the loop, and strict.** Unknown, missing,
  duplicate or out-of-bound members refuse the start, naming the path
  (jig's `protocol/configuration.md`, section 1). A change is a restart.
- **jig's parts are jig's:** `jig-protocol-config` parses them; temper
  writes no parser of its own for them, and adds only the checks
  standalone needs (section 2).
- **temper's members:** `forges`, `identities`, `landing`, `charter`,
  `clients` and `limits` for the engine (section 3), and `workers` for
  each worker (section 4). `webhooks` is reserved, and refused until it
  is built.
- **Not configuration:** repositories, which people adopt at runtime;
  the branch namespace, which the root derives from the deployment's id;
  the forge writer's identity, which comes from each forge's `user`.
- **No secret in the document.** Each secret is a private file the
  document names, read at start (section 6).
- **One format rule** for temper's typed members: member names are the
  Rust field names, and a plan writes the parser from the types (section
  10).
- **Standalone shape:** exactly one worker, joined to the engine by the
  in-process name `workers`; the clients' listener on loopback; the
  agent program is temper's own binary.
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
  section 3): `deployment`, `owners`, `providers`, `rules`, `projects`,
  `charters`, `templates`, `hosts`, `llm`, `clients`, `connectors`,
  `limits` and `memory_bytes`, in jig's formats (jig's
  `protocol/configuration.md`, section 10). Its `clients` member is what
  jig's clients' component takes; the clients' listener is temper's
  (section 3).
- **`workers`** is a list of jig's worker parts (jig's
  `protocol/configuration.md`, section 4): `engine`, `name`, `secret`,
  `slots`, `graces`, `agent`, `limits` and `memory_bytes`.
- **`application`** holds temper's members (sections 3 and 4).
- **jig's reference fixtures** are a complete example of jig's parts: the
  reference binary's test configuration (`jig-test`'s
  `tests/support/mod.rs`, made from jig's channel world's `fixtures.rs`).
  temper's process tier writes its own the same way (testing.md, 2.6).
- **What standalone requires of jig's parts,** each checked at start, a
  failure naming its path:
  - `workers` has exactly one entry, whose `engine` is `{"in_process":
    "workers"}`;
  - `engine.hosts.listener` is `[{"in_process": "workers"}]`, and
    `engine.hosts.workers` names that one worker;
  - `engine.connectors` is exactly `[1]`: the forge connector, temper's
    one connector, whose number both ends of the client's wire fix as
    `temper-client-protocol-wire`'s `FORGE` (client.md, 4.6);
  - the worker's `agent` paths are absolute (section 5).

  The worker's `agent` names temper's own binary (section 5), which no
  check can tell from another program.
- **Templates are jig's two defaults,** chat (template 1, label 1) and
  goal (template 2, label 2), as jig's `protocol/configuration.md`,
  3.1 gives them, the chat's executor the charter chats run under;
  temper defines no template of its own. Its client's lists filter on
  those labels (client.md, 3.5), and a project's policy may override
  their defaults as jig's allows.
- **Reading,** by `temper serve`, in this order:
  1. the file, whole, with skein's startup kit, within the bound jig's
     reference binary sets on a configuration's bytes;
  2. jig's parts, by `jig-protocol-config`; `application`, by
     `temper-service`'s startup, with `jig-protocol-config`'s readers, so
     every problem names its path the same way
     (`application.forges[0].user`);
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
| `user` | u64 | above 0 | the forge user the API token belongs to: the forge writer's identity, checked at start (credentials.md, section 5) |
| `identity` | u32 | the `number` of one entry of `identities`; each identity named by exactly one forge | the git identity this forge's runs use (credentials.md, section 4) |

**`identities`:** a list of git identities, temper's application
credentials (credentials.md, section 4), as many as `forges`, and at most
jig's limit on an engine's application credentials (the accounts child's
`credentials`). Each:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `number` | u32 | unique among `identities` | the credential's number, as the root names it in `Event::Credential` and in grants |
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
| `enforce` | boolean | true makes each criterion below a requirement every landing it covers must meet (`domain/authority.md`, section 10); false leaves them to the change procedure alone |
| `ci` | boolean | CI must have passed at the change's head |
| `up_to_date` | boolean | the change's head must contain its base's tip |
| `gates` | list of `{"number": u32, "blocking": boolean, "freshness": "exact" or "clean"}` | the gates every change it covers asks for (`domain/forge.md`, 8.3): an agent's review, or one of a repository's checks when it has no CI; at most the forge's `change_policy.gates` |
| `approvals` | list of `{"role": u32, "people": u32, "freshness": "exact" or "clean"}` | `people` distinct people holding `role` in the project approve the change |

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
(client.md, section 8):

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `listener` | an address, jig's format | a socket address; on loopback (in `127.0.0.0/8`, or `::1`) in standalone, and wherever `engine.clients` enables the printed link (jig's `protocol/service.md`, section 3) | the listener on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig`; the mapping's Host allow-list; and the address of the sign-in links `temper serve` prints (client.md, section 7) |

**`limits`:** temper's profile and overrides (section 7).

**`webhooks`:** reserved. Present, it refuses the start as not built
(forge.md, section 5).

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
`kind`, `api`, `transport`, `api_token` and `user`; the identities'
table takes each identity's value; the engine's routing takes each
forge's `git`; the clients' listener takes `clients.listener`.

## 4. temper's members for a worker

**`application.workers`:** a list with one entry per entry of the
top-level `workers`, in the same order; standalone has one. Each:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `root` | a directory | made private (0o700) if absent; an existing one must be the process's user's, private, and not reached through a link | the worker's workspace directory: every checkout's workspace is made under it (git.md, section 2; `domain/worker.md`, section 5) |
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

  so each agent process is `temper agent AGENT.json` (protocol.md,
  section 4).
- **Read twice:** by `temper serve` at start, for smith's host limits,
  the channel's limits and the agent's bounds, as jig's reference binary
  reads it; and by each agent process as it starts.
- **It holds no credential:** an agent's grants come over smith's
  channel (credentials.md, section 3).
- **The same names as the engine's,** checked at start: every endpoint of
  `engine.llm.endpoints` must be in the agent's configuration under the
  same name with the same account, and every model a charter names must
  be declared there, or the start is refused, naming it. One process
  reads both, so a mismatch that would fail every run's charter is
  caught before the loop (llm.md, section 3).

## 6. Secrets' files

| File | Named by | Holds | Read by |
|---|---|---|---|
| a forge's API token | `application.forges[].api_token` | the token's bytes | the forge component |
| a git identity's token | `application.identities[].token` | the token's bytes | the identities' table |
| a worker's secret, its digest, a handed-in LLM account's file, the accounts' records' directory, an LLM account's client secret | jig's parts | jig's formats (jig's `protocol/configuration.md`, sections 7 and 10) | jig's components |

- **A secret file holds its bytes and nothing else** but a final
  newline, which is not part of the secret (jig's
  `protocol/configuration.md`, section 10).
- **Read at start,** each within its bound, with skein's
  `read_private_file` (skein's `shell.md`, 6.1): a file readable by
  anyone but the process's user, or reached through a link, refuses the
  start, naming it. An API token's bound is the forge component's
  `token_bytes` (forge.md, section 7); an identity's token's is the
  channel's bound on a grant's value, which its whole value, login and
  author included, must fit (section 3; channel.md, section 8).
- **Never written back.** temper writes none of them; an operator
  replaces one and restarts (credentials.md, sections 4 and 5).
- **No temper secret in jig's secrets' store** (credentials.md, section
  6).

## 7. Limits

```
"limits": {"profile": "small", "overrides": {"<name>": value}}
```

- **Every limit temper's own domains and components take:** the forge's
  and its change and issue policies, the queue window, the forge
  client's, the checkout's, the root's own beyond jig's core, the forge
  component's and the git component's.
- **One profile, `small`,** for one person's machine; another name is
  refused. Its numbers are constants in `temper-service`'s limits module,
  each documented with what it bounds. Where a limit of temper's is a
  quantity jig's profile also sets, its constant cites jig's small
  profile's (jig's `protocol/configuration.md`, section 6).
- **Overrides by dotted name:** the path of the field in its owner's
  `Limits` type, a duration's last segment with `_ms`. A name temper does
  not know, or a value outside its type, refuses the start, naming it.

  | Prefix | Owner |
  |---|---|
  | `root.` | the engine root's own limits, beyond those jig's `engine.limits` sets |
  | `forge.` | `temper-engine-domain-forge`'s `Limits`: `forge.client.`, `forge.change_policy.` and `forge.issue_policy.` for its children, `forge.queue_window_ms` |
  | `checkout.` | `temper-worker-domain-checkout`'s `Limits` |
  | `forge_component.` | the forge component's (forge.md, section 7) |
  | `git_component.` | the git component's (git.md, section 8) |

  For example `forge.repositories`, `forge.client.calls`,
  `checkout.remote_timeout_ms`.
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
  - the channel's (channel.md, section 8): each of temper's largest
    parts within the ceiling jig's schema gives its field; the largest
    part the engine can send within what the worker's limits take; the
    longest remote the routing can make (the longest `git` base, an
    owner and a name of the forge's `name_bytes`, two `/` and `.git`),
    and the forge's names and branches, within the checkout's
    `name_bytes`.
- **jig's limits stay in jig's parts:** `engine.limits` and each worker's
  `limits` (jig's `protocol/configuration.md`, section 6).
- **The engine's documents fit temper's client.** A document past the
  bounds a client decodes is refused there whole, as too large (jig's
  `protocol/client.md`, section 3), so the documents the engine serves
  are within what temper's client takes: its client takes `jig-client`'s
  limits from jig's small profile with the same `documents.` overrides
  the example's `engine.limits` names (client.md, section 2). The engine
  cuts each text to its bound and says so (jig's
  `client/domain/contract.md`, 5.3). It checks none of this at
  start: it serves whatever client the deployment lets in, whose limits
  are the client's. A test holds the example's and the process tier's
  configuration to temper's client's limits, so the two cannot drift
  apart unseen.
- **The forge's paths fit a document.** jig's documents carry a
  resource's path and a pattern whole, with as many segments as the
  engine admits in a name (jig's `client/domain/contract.md`, 5.3), and
  `documents.lines` bounds a document's short lists alone, at most the
  client's rows. So `engine.limits` names the bounds on a name's
  segments, `authority.segments`, `tasks.hold_segments` and
  `tasks.authority_segments`, at least the segments of the deepest path
  the forge names: a branch's, the forge's five segments then the
  branch's own (`domain/forge.md`, section 2), 8 as temper's branches
  name them. The example names them 8. temper's client takes a path's
  segments, 9, apart from its rows (jig's `client/domain/client.md`,
  section 14), so a page of the forge's work is never refused for a
  path, and a long path shows cut, with jig's mark (client.md, 5.4).
- **One worst case per service:** jig's sum (the root's, its components'
  and the journal's) and temper's (its domains', its components', the
  identities' table) together must fit the memory jig's part states,
  `engine.memory_bytes` for the engine's service and `memory_bytes` of
  the worker's part for the worker's, or the start is refused, saying by
  how much.

## 8. First start and every start

- **A deployment's first start** is the first with an empty store (jig's
  `protocol/configuration.md`, section 8): the shell draws the
  deployment's id, and the core seeds owners and projects from
  `engine`. temper adds:
  - **periods from the smallest source,** as jig's reference service
    gives them (jig's `protocol/configuration.md`, section 5): before the
    core admits a party's request, the engine's routing gives the root
    one period for each configured project, its budget the project's
    spend per period, with no next start;
  - **each project's landing rules** join its policy as it is seeded.
- **Every start:**
  - jig's parts apply as jig says: its rules, charters, limits, hosts,
    endpoints and accounts as configured, its projects ignored;
  - temper's members apply as configured: the forges, the identities,
    the deployment's landing rules, the forge's judges for every rule,
    the charter, the limits and the workers;
  - every limit is checked again, temper's and jig's together, with the
    channel's start checks on temper's parts (section 7; channel.md,
    section 8), so a `git` base or a limit changed since the last start
    is refused before io starts when a remote or a part could pass its
    bound;
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
  client.md, section 4).
- **What a change reaches,** after its restart: a forge's `git`, every
  remote from the next assignment on; a forge's `api_token` and `user`,
  checked again; an identity's token, granted again at generation 1.

## 9. The client's arguments

- **`temper open DIRECTORY LINK_OR_ADDRESS`** takes no configuration
  file (client.md, 6.1 holds its arguments exactly):
  - `DIRECTORY` is the client's private directory, made 0o700 if absent,
    and refused unless it is a directory of the user's with no group or
    other permission bits (skein's `io.md`, 5.3). It holds the bearer
    token and the small store, `jig-client-shell-native`'s files (jig's
    `protocol/client.md`, sections 8, 10 and 12; client.md, section
    7).
  - `LINK_OR_ADDRESS` is the link `temper serve` printed, the first
    time, by which the client signs in (jig's `protocol/client.md`,
    section 8; client.md, section 7); or the clients' listener's
    `http://HOST:PORT`, later, when the directory holds a bearer token.
  - HOST must be an IP address on loopback, in standalone; another is
    refused before anything is sent.
  - The client's limits are constants of temper's client (client.md,
    section 2).
- **`temper sign-in CONFIG.json ACCOUNT`** reads the same document as
  `temper serve` (credentials.md, section 3).
- **`temper agent AGENT.json`** reads smith's agent configuration
  (section 5).

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
    "identity": 1
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
  "limits": {"profile": "small", "overrides": {}},
  "workers": [{"root": "workspaces", "git": "/usr/bin/git"}]
}
```

A landing rule may name a repository no project has adopted yet: it
covers nothing until a person adopts `acme/widget`.

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
    "limits": {"profile": "small", "overrides": {"authority.segments": 8, "tasks.hold_segments": 8, "tasks.authority_segments": 8}},
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
    "limits": {"profile": "small", "overrides": {}},
    "memory_bytes": ...
  }],
  "application": { ... as above ... }
}
```

## 11. Open questions

- **A calendar for periods.** The smallest source opens one period per
  configured project at the first start and none after, so a project's
  budget is not renewed. Default: as is, until a calendar child of
  temper's root is designed, as `ops`'s is (jig's
  `protocol/service.md`, section 7).
- **`webhooks`' member.** Default: reserved and refused; when built, each
  forge's webhooks' listener address and secret file, by forge.md,
  section 5.
- **A forge removed while the store holds its repositories.** Default:
  the start is refused at the forge's restart step, naming the forge
  number (jig's `protocol/configuration.md`, section 9, removed parts).
- **A project's landing rules changed after its first start.** They
  reach the forge's judges at the next start, but the project's policy
  keeps the requirements seeded at its first start. Default: a change
  reaches the policy only through a policy change by a person.
- **A repository renamed or moved after adoption.** Default: its
  resource path, which landing patterns match, and its remote keep the
  owner and name of its adoption.
- **Profiles' numbers,** from the first deployments' measurements
  (jig's `protocol/configuration.md`, section 9). Default: `small` as
  written.
- **A configuration's format version** (jig's open question). Default:
  none; unknown members refuse the start.
