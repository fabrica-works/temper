# Credentials

Provisional, 2026-10-03, revised 2026-10-19. temper's credentials: which
secrets temper's processes hold, where each comes from, who may use it,
and how it reaches the process that does. LLM accounts, workers' secrets
and people's sign-ins are jig's (jig's `protocol/README.md`, section 8;
jig's `domain/accounts.md`; jig's `protocol/llm.md`, sections 5 and 6),
and temper adds `temper program-login` (section 3); temper's own are
its git identities, the forges' API tokens and their webhooks' secrets.
The files they come from are configuration.md's (section 6); a git
identity's value on the channel is channel.md's (section 6); git's use
of it is git.md's (section 5); the credentials of the forge's setup,
which `temper serve` never holds, are setup.md's (section 9). What is
open is listed in section 8.

## 1. In one page

- **Names through the domains, values beside them.** A domain names a
  credential by its number and a generation, and never holds its value,
  so no trace, snapshot, record or log of a domain can leak one. Values
  live in the protocol layer's tables, filled in as a record is encoded
  and taken out as it is decoded.
- **LLM accounts are jig's,** made from the client: a sign-in with the
  LLM provider, a key, or a program's login. temper writes no OAuth
  exchange and configures no account, and adds `temper program-login`,
  which signs a program's login in on a host (section 3).
- **Git identities are temper's application credentials,** one per
  forge: a forge token, a login and an author, read from a private file
  at start, granted with each assignment that needs one. Each
  repository's runs use its forge's git identity (section 4).
- **The API token is the engine's protocol layer's only.** One per forge,
  read at start, never granted, never stored, and checked against the
  forge's user before temper acts as it (section 5).
- **The webhooks' secret is the forge component's too.** One per forge
  with webhooks, read at start, by which the webhooks' listener checks
  each delivery's signature; never granted, never stored, and never sent
  by the engine (forge.md, section 5). Reconcile gives it to the hooks it
  makes (setup.md, section 5).
- **Git gets its credentials per invocation,** in that process's
  environment only, never in an argument, a URL or a file.
- **Least power.** The git identity is apart from the API token: a worker
  needs only fetching and pushing, so a deployment can give git a token
  with less power than the engine's. The engine never holds a site
  admin's token: when temper sets a forge up, the admin's token is read
  by `temper forge reconcile` alone, which ends (setup.md, section 9).
- **No temper secret in jig's secrets' store** (section 2).
- **Workers keep nothing between runs.** A worker's configuration holds
  no credential but its own secret; every other comes with assignments.

## 2. The credentials

| Credential | Owner | Kind | Source | Held by | Used by | Changes |
|---|---|---|---|---|---|---|
| an LLM account's source | jig | a signed-in record (refresh token, access token, expiry), a key, or a program's login | a sign-in or a key from the client; a program's login, `temper program-login` on each host (section 3) | jig's secrets' store, lent by skein's accounts driver, the access tokens and keys in jig's accounts' table; a program's login, the program, in the source's configuration directory | the worker's agent processes, through grants | the driver refreshes a signed-in record; a key's holder replaces it from the client; the program renews its own login |
| a git identity | temper | a forge token, a login, an author | a private token file per identity, the person's, or reconcile's when its forge is managed or owned (setup.md, section 5), and the identity's members | the identities' table on the engine, and on the worker from grants | the git component's invocations | an operator replaces the file and restarts; reconcile makes it again at a start when its file does not answer as the bot |
| a forge's API token | temper | a forge token | a private file per forge, the person's, or reconcile's when the forge is managed or owned (setup.md, section 5) | the forge component | that forge's API requests | an operator replaces the file and restarts; reconcile makes it again at a start when its file does not answer as the bot |
| a worker's secret | jig | a shared secret | the worker's secret file; the engine's digest file | the worker; the engine, its digest | the link's opening | an operator replaces both |
| a sign-in's secret | jig | a bearer token | the printed link, then the sign-in | `jig-client` in `temper open`, which keeps it in its directory; the engine, its digest | each of the client's exchanges and its stream | the sign-in ends or expires |
| a forge's webhooks' secret | temper | a shared secret | a private file per forge, `forges[].webhook_secret`: the person's, or reconcile's, drawn when the file is missing (setup.md, section 5) | the forge component's webhooks' listener, on loopback; the forge's hooks, which sign each delivery with it | checking each delivery's signature (forge.md, section 5) | an operator replaces the file and the hooks' secret, and restarts; reconcile patches its hooks with the file's at every start |

- **No temper secret in jig's secrets' store** in standalone: every
  temper secret is a file read at start. jig keeps sign-ins' and
  services' digests there, and LLM accounts' records and keys (jig's
  `protocol/store.md`, section 9).
- **The forge's setup's credentials** (a site admin's token, the bot's
  and temper's admin's passwords, the person's first password on an
  owned Forgejo, the runner's secret, Forgejo's own secrets) are
  setup.md's (section 9): made and read by the setup's commands, kept in
  the forge's directory or nowhere, and never held by `temper serve`.

## 3. LLM accounts

jig's, as jig has them (jig's `domain/accounts.md`): each an LLM
provider's account, held by a person or by the deployment, with its
sources in its holder's order. temper configures the LLM providers
(configuration.md, section 2), shows and makes accounts from its client
(`../client/README.md`, 3.5; `../tools.md`, 5.5, area 5), and adds
`temper program-login`, below.

- **Made from the client, never configured.** Configuration names LLM
  providers, never an account (jig's `protocol/configuration.md`, section
  3). A fresh deployment has no account but a *none* source's, so no run
  on another LLM provider starts until a person adds one, and the client
  says so (`../client/README.md`, 5.1).
- **Kept by jig:** a signed-in record and a key in jig's secrets' store
  (jig's `protocol/store.md`, section 9), lent by skein's accounts driver
  through `jig-protocol-accounts` (jig's `protocol/llm.md`, section 5);
  never in a root, temper's records, a fact, a request or an answer. A
  key reaches the engine in a request the client sends once and never
  keeps (jig's `domain/accounts.md`, section 5).
- **The sources** (jig's `domain/accounts.md`, 3.4):
  - **signed in:** the person's sign-in with the LLM provider, asked
    from the client. The engine binds a loopback listener for each
    sign-in, on an ephemeral port unless the LLM provider's
    configuration fixes one (jig's `protocol/llm.md`, section 5; skein's
    `oauth.md`, 6.5), so the browser that approves must run on the
    engine's machine. The client shows the address to open; the terminal
    prints it and opens no browser (`../client/README.md`, 6.5);
  - **a key:** pasted on the accounts page, or read by `temper ctl` from
    a private file;
  - **a program's login:** declared from the client under its name, then
    signed in on each host by `temper program-login` (below). The login
    is the program's own, in the source's configuration directory,
    `<directory>/NAME` under the agent's programs directory
    (configuration.md, section 5), and temper never reads it;
  - **none:** an LLM provider whose endpoint takes no credential, such
    as a local model's on loopback; its one account is the deployment's.
- **The engine makes no LLM call** (llm.md, section 2): only agent
  processes use an account's values.
- **On the worker,** the routing puts each grant in smith's form (jig's
  `protocol/llm.md`, section 6) and hands it to smith's host half, which
  fills it into the agent's start and grants: a credential's value with
  its scheme, a bearer or a key; or a program's source by its name, by
  which the agent finds its configuration directory (smith's
  `protocol/programs.md`, section 3). Limited, refused and unauthorized
  travel back from the agent as notices, through smith's host half, the
  hub and the link, to the accounts child, fenced by attempt (jig's
  `domain/accounts.md`, 3.5).
- **A host that keeps a person's program logins** runs only that
  person's work, until a run's process tree is contained from the
  programs directory (jig's `domain/accounts.md`, 3.4): a run can read
  every login kept on its host. A standalone deployment's one worker,
  on its person's machine, keeps that person's logins alone, and is
  unaffected.
- **`temper program-login CONFIG.json NAME`:**
  - `NAME` is a program source's name, 1 to 32 of `a` to `z`, `0` to `9`
    and `-` (jig's `domain/accounts.md`, 3.4), as the accounts page and
    `temper ctl declare` give it with this command.
  - It reads and checks `CONFIG.json` as `temper serve` does
    (configuration.md, section 8), and the agent's configuration it
    names, reading no secret file; it opens no store, starts no service
    and sends nothing to the engine, which need not run.
  - It refuses, exit 1, before it makes anything: a name out of its
    bounds; an agent's configuration with no program (smith's
    `protocol/agent.md`, section 4); a programs directory, or a
    `<directory>/NAME`, that exists and is not a directory of the
    user's own with no group or other bits, or is reached through a link
    (skein's `io.md`, 5.3).
  - It makes the programs directory and `<directory>/NAME`, each
    `0o700`, when missing, then replaces itself with the program's own
    sign-in, `claude auth login`, at the configured path. The
    environment is the caller's, so that the program's sign-in has the
    person's terminal and browser, with `CLAUDE_CONFIG_DIR` the source's
    directory and the program's credential variables removed, as every
    command of a program source's is (smith's `protocol/programs.md`,
    4.2): `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`,
    `CLAUDE_CODE_OAUTH_TOKEN` and the cloud providers' variables, which
    would win over the login (jig's `domain/providers.md`, 3.3). The
    forge's commands clear the environment before they exec (setup.md,
    6.4); a sign-in cannot.
  - Its exit status is the program's. A failure before the exec exits
    1, its reason on stderr, one line beginning `temper program-login:`.
  - It reads nothing of the login. The host's half checks it (smith's
    `protocol/programs.md`, section 3; smith's `protocol/hosts.md`,
    section 2) at its hello, when told of a new source, on its holder's
    check and at its interval (jig's `domain/accounts.md`, 3.4), and the
    accounts page shows the state.
  - Its name says what it does, never a provider's product (jig's
    `domain/providers.md`, 3.2).
  - A worker on another host is signed in on that host, with that host's
    configuration.
- **A program's sign-out** is the host's half's, on each host, by the
  program's own (smith's `protocol/hosts.md`, section 2; jig's
  `domain/accounts.md`, 3.4), when its holder signs the source out or
  removes it from the client.
- **What temper does not do:** configure an account, sign one in at a
  terminal of its own, keep a record or a key in its own store or a
  file, refresh an account, or read a program's login.

## 4. Git identities

- **What one is:** a forge token git authenticates with, the forge login
  it goes with, and the author (a name and an email) of the commits made
  with it. It is an application credential (jig's `domain/engine.md`,
  12.1), numbered by configuration (configuration.md, section 3).
- **Each repository to a git identity.** Each forge names one identity,
  and each repository's runs use its forge's git identity: the root's
  `GitIdentity { forge, credential }` comes from configuration, and each
  repository of an assignment's workspace names its forge's. An identity
  serves exactly one forge.
- **Its source** is the private token file `identities[].token`, read at
  start; its login and author are the identity's members. Each start
  gives it generation 1. Its validity is unbounded (`Duration` at its
  largest, as jig's reference service gives its credential): Forgejo's
  tokens do not expire.
- **Announced first.** The engine's routing gives the root
  `Event::Credential { number, generation: 1, valid }` for each identity
  as its first events, before its restart, so every identity is usable
  when decisions open (jig's `domain/engine.md`, 12.1;
  `../domain/root.md`, 7.1).
- **The identities' table, on the engine:** each identity's value,
  temper's record of its token, login and author (channel.md, section 6),
  encoded once at start by `temper-protocol-worker-wire`, and held as one
  secret by number and generation. No root, store record, fact or log
  ever holds it.
- **Granted with the assignment.** A run whose workspace names a
  repository is granted its forge's identity, the grant naming the
  application's table; `jig-protocol-engine-workers` fills the value in
  from the identities' table, through the routing, as it encodes the
  assignment or a grant record, and grants follow their attempts as an
  account's do (jig's `protocol/channel.md`, section 8). A run with no
  repository is granted none.
- **On the worker,** `jig-protocol-worker-link` takes the value out into
  the worker's identities' table, by number, at most the two newest
  generations; the hub hears the name only. The routing hands the git
  component the identity's newest value with each operation that names
  it, and the component holds it only while that operation is in flight;
  it never reads the table itself (git.md, sections 2 and 5). An
  operation whose identity has no value in the table fails `Broken`
  without running git. An entry goes as jig's do (jig's
  `protocol/channel.md`, section 8); a git identity's never expires, so
  only a newer generation pushes it out.
- **In git,** the environment only: an `http.extraHeader` of HTTP basic
  authentication from the login and the token, and the author's
  variables; never an argument, a URL, a repository's configuration or a
  file (git.md, section 5).
- **Agents never get one.** An agent process's environment holds no
  credential (jig's `protocol/service.md`, section 6), and the hub hands
  an agent its LLM grants only.
- **A forge that refuses the identity** fails the workspace, never the
  credential: a preparation it refuses is `Preparation::Refused`
  (`../domain/worker.md`, 4.3), a push `Landing::Refused`
  (`../domain/worker.md`, section 5). Limited, refused and unauthorized
  are LLM accounts' notices only (jig's `domain/accounts.md`, 3.5).
- **Replaced** by an operator, who replaces the token file and restarts;
  the new start grants generation 1 again. In standalone no worker
  outlives the engine, since the worker is in its process, so no holder
  can mix the two values under one name (section 8).

## 5. The forges' API tokens

- **One per forge,** read at start from `forges[].api_token`, and held
  by the forge component alone. It goes in the `Authorization: token`
  header of each request to that forge's API (forge.md, 3.1), over TLS
  unless the forge is on loopback (configuration.md, section 3).
- **Never granted, never stored:** not in a grant, a root, the store, a
  fact or a log, and never sent to another forge or another host.
- **Checked at start.** The forge component's first exchange with each
  forge is `GET /api/v1/user`; the answer's `id` must equal
  `forges[].user`: the number it gives, or the id in decimal in the
  private file it names, read at start (configuration.md, sections 3 and
  6), which reconcile writes when the forge is managed or owned
  (setup.md, section 5). Another id, or a `401` or `403`, stops the
  engine, naming the forge, whether at start or when the forge first
  answers: a wrong token would make temper act as someone else, so the
  check is fail-stop, never a degraded mode (README.md, section 6).
  Until it passes, the forge's calls wait in the component, and a check
  that goes unanswered is made again (forge.md, 3.5).
- **The same user is the forge writer** in the domain (`Writer { forge,
  author: user }`), by which the forge connector tells temper's own
  writes from others': a key's provenance, and its own writes' echoes
  (`../domain/forge.md`, 5.2; forge.md, section 4).
- **A token the forge stops taking** fails each call as forge.md, 3.4
  classifies a refusal; nothing refreshes it while the engine runs. An
  operator replaces the file and restarts, and the check runs again; on
  a managed or owned forge, reconcile makes it again before the restart's
  `temper serve` (setup.md, 7.2 and section 5).

## 6. Workers' secrets and sign-ins

- **A worker's secret is jig's** (jig's `protocol/channel.md`, 4.1). In
  standalone the link is in process, and still opens with the worker's
  name and secret; the files are jig's worker and engine parts'
  (configuration.md, section 6).
- **People's credentials are jig's** (jig's `protocol/client.md`,
  section 8): jig mints, verifies, expires and revokes them. Its printed
  link signs in the deployment's first owner: `temper serve` prints the
  link at start, `temper open` signs in by it; its client holds the
  bearer token it is given and sends it with every exchange, and the
  terminal keeps it in its private directory, to sign in with it again
  (`../client/README.md`, section 7). Every other way of signing in is open
  (section 8).
- **No TLS key** in standalone: nothing temper serves leaves the machine.

## 7. Testing

- **No value across a domain boundary:** the one-process world and the
  forge's protocol world search every root's events and requests, every
  fact and every store record for the values their fakes issued (LLM
  access tokens and keys, identities' tokens, API tokens, webhooks'
  secrets), and fail on any (`../testing.md`, 2.5).
- **The identities' table:** step tests of its filling at start, a value
  filled into a grant and taken out on the worker, and two generations.
- **Git's environment:** step tests of its builder: the header's bytes,
  the author's variables, and no credential in an argument or a URL
  (git.md, section 9).
- **The user check,** in the forge's protocol world: a token of another
  user stops the engine, naming the forge; calls wait until the check is
  answered; an unanswered check is made again.
- **Startup:** a secret file readable by another user, or reached
  through a link, refuses the start, naming it.
- **`temper program-login`,** step tests: each refusal (a name out of
  its bounds, no program configured, a directory not private or reached
  through a link), each before anything is made; the directory made
  `0o700`; and the exec's program, arguments and environment: the
  caller's kept, `CLAUDE_CONFIG_DIR` the source's directory, the
  program's credential variables removed. The process tier execs smith's
  fake `claude` (smith's `protocol/programs.md`, section 8) and finds its
  login in the source's directory (`../testing.md`, 2.6).
- **LLM accounts' stories** in the real loop (`../testing.md`, 2.4): a
  key added, a sign-in against skein's fake issuer with a redirect the
  test drives, and a program's source over smith's fake `claude`; after
  each, no value in temper's store, a fact, a request, an answer or the
  client's small store.
- **The live tier** runs on a program's login on its machine, which
  temper never reads, or on a sign-in or a key added per run
  (`../testing.md`, 2.7).

## 8. Open questions

- **Git's environment, readable by agents.** An agent run as the same
  user could read a git process's environment. Default: accepted in
  standalone, where every process is the person's; between hosts,
  contained process trees give agents a process namespace of their own,
  which closes it (jig's `protocol/README.md`, section 10). Without
  them, the token would reach git through a pipe and a credential helper
  instead. An owned runner's jobs run as the same user too; their
  isolation, CI's, is setup.md's (section 15).
- **Generations across starts.** Generation 1 at every start names a new
  value once a token file is replaced, which is sound only while no
  worker outlives the engine. Default: standalone only; workers on other
  hosts wait for a generation kept across starts.
- **Short-lived git credentials.** Forgejo's tokens do not expire. A
  forge that issues short-lived ones (GitHub's app installation tokens)
  would make a git identity's source an exchange the protocol layer
  refreshes, announcing each new generation (jig's `domain/engine.md`,
  12.1). Default: static tokens only.
- **A second program.** `temper program-login` signs in the agent's
  one program, Claude Code. Default: it takes `--program ADAPTER` once
  smith's agent adapts a second (smith's `protocol/programs.md`).
- **The forge as people's sign-in provider,** for a deployment with more
  than one person: routes of temper's on its clients' listener run the
  forge's OAuth flow, read who the person is, and ask jig's clients'
  component to mint a credential for that identity, with a `Mint` from
  temper's server code alone (jig's `protocol/client.md`, section 8;
  jig's `protocol/service.md`, section 2; `../client/README.md`, section 11).
  The forge's OAuth client secret is then temper's, a private file read
  at start. Default: the printed link signs in the deployment's one
  owner.
