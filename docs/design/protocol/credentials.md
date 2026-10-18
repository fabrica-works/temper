# Credentials

Provisional, 2026-10-03, revised 2026-10-18. temper's credentials: which
secrets temper's processes hold, where each comes from, who may use it,
and how it reaches the process that does. LLM accounts, workers' secrets
and people's sign-ins are jig's (jig's `protocol/README.md`, section 8;
jig's `protocol/llm.md`, sections 5 and 6); temper's own are its git
identities, the forges' API tokens and their webhooks' secrets. The files
they come from are configuration.md's (section 6); a git identity's value
on the channel is channel.md's (section 6); git's use of it is git.md's
(section 5); the credentials of the forge's setup, which `temper serve`
never holds, are setup.md's (section 9). What is open is listed in
section 8.

## 1. In one page

- **Names through the domains, values beside them.** A domain names a
  credential by its number and a generation, and never holds its value,
  so no trace, snapshot, record or log of a domain can leak one. Values
  live in the protocol layer's tables, filled in as a record is encoded
  and taken out as it is decoded.
- **LLM accounts are jig's,** on skein's accounts driver; temper writes
  no OAuth exchange, and adds `temper sign-in`, an account's sign-in at a
  terminal (section 3).
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
| an LLM account | jig | an OAuth record (refresh token, access token, expiry), or a handed-in access token | `temper sign-in`, or a private file another program's login fills | skein's accounts driver; the access tokens lent, jig's accounts' table | the worker's agent processes, through grants | the driver refreshes a signed-in one; a handed-in one is replaced by hand, then read at the next start |
| a git identity | temper | a forge token, a login, an author | a private token file per identity, the person's, or reconcile's when its forge is managed or owned (setup.md, section 5), and the identity's members | the identities' table on the engine, and on the worker from grants | the git component's invocations | an operator replaces the file and restarts; reconcile makes it again at a start when its file does not answer as the bot |
| a forge's API token | temper | a forge token | a private file per forge, the person's, or reconcile's when the forge is managed or owned (setup.md, section 5) | the forge component | that forge's API requests | an operator replaces the file and restarts; reconcile makes it again at a start when its file does not answer as the bot |
| a worker's secret | jig | a shared secret | the worker's secret file; the engine's digest file | the worker; the engine, its digest | the link's opening | an operator replaces both |
| a sign-in's secret | jig | a bearer token | the printed link, then the sign-in | `jig-client` in `temper open`, which keeps it in its directory; the engine, its digest | each of the client's exchanges and its stream | the sign-in ends or expires |
| a forge's webhooks' secret | temper | a shared secret | a private file per forge, `forges[].webhook_secret`: the person's, or reconcile's, drawn when the file is missing (setup.md, section 5) | the forge component's webhooks' listener, on loopback; the forge's hooks, which sign each delivery with it | checking each delivery's signature (forge.md, section 5) | an operator replaces the file and the hooks' secret, and restarts; reconcile patches its hooks with the file's at every start |

- **No temper secret in jig's secrets' store** in standalone: every
  temper secret is a file read at start. jig keeps sign-ins' and
  services' digests there (jig's `protocol/store.md`, section 9).
- **The forge's setup's credentials** (a site admin's token, the bot's
  and temper's admin's passwords, the person's first password on an
  owned Forgejo, the runner's secret, Forgejo's own secrets) are
  setup.md's (section 9): made and read by the setup's commands, kept in
  the forge's directory or nowhere, and never held by `temper serve`.

## 3. LLM accounts

jig's, as jig has them: configured in jig's `engine.llm` (jig's
`protocol/configuration.md`, section 3), held on skein's accounts driver
and lent by `jig-protocol-accounts` (jig's `protocol/llm.md`, section 5),
granted with each assignment and pushed to every live attempt (jig's
`protocol/llm.md`, section 6). temper adds no member, and:

- **The engine makes no LLM call** (llm.md, section 2): only agent
  processes use an account's tokens.
- **On the worker,** the routing puts each value in smith's form (jig's
  `protocol/llm.md`, 5.3) and hands it to smith's host half, which fills
  it into the agent's start and grants. Rejected and exhausted travel
  back from the agent through smith's host half, the hub and the link to
  the accounts child, fenced by attempt.
- **An account's first record,** one of two (jig's `protocol/llm.md`,
  5.4):
  - **handed in:** a private file in jig's format, which the driver
    lends until it lapses, never refreshing it and never writing its
    source. The live tier hands in a borrowed Codex login's access token
    so (`../testing.md`, 2.7);
  - **signed in:** by `temper sign-in`, below.
- **`temper sign-in CONFIG.json ACCOUNT`:**
  - `ACCOUNT` is the `number` of one of `engine.llm.accounts`.
  - It reads and checks `CONFIG.json` as `temper serve` does
    (configuration.md, section 8), reading no secret file but the
    account's own; it opens no store and starts no service. The engine
    need not run.
  - It refuses, before anything else and naming the account: a number no
    account has; a handed-in account; and an account whose registration
    has a client secret, since a sign-in at a terminal serves a public
    client only (jig's `protocol/llm.md`, 5.4).
  - It runs skein's accounts driver's sign-in (skein's `oauth.md`,
    sections 2 and 6): it prints the visit's URL on stdout, one line;
    waits for the provider's redirect on the registration's loopback
    listener, within the driver's sign-in deadline (skein's `oauth.md`,
    section 3); exchanges the code; keeps the record in the account's
    private file; and exits 0.
  - It never opens a browser, and prints no secret.
  - It exits 1 on any failure (a refusal, the deadline passed, the
    provider's error, a record not kept), the reason on stderr.
  - A running engine loads the new file at its next grant of the
    account, since its driver loads an account's file again whenever it
    holds no record of it (jig's `protocol/llm.md`, 5.4): no restart.
  - It is copied from smith's local shell's sign-in (smith's `shell.md`,
    4.5 and section 7), on the same driver.
- **What temper does not do:** refresh an account itself, keep an
  account's record in its store, or sign an account in through the
  client (jig's `protocol/llm.md`, section 10).

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
  (`../domain/worker.md`, section 5). Rejected and exhausted are LLM
  accounts' only (jig's `domain/engine.md`, 12.1).
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
  access tokens, identities' tokens, API tokens, webhooks' secrets), and
  fail on any (`../testing.md`, 2.5).
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
- **`temper sign-in`,** against skein's fake issuer (skein's `oauth.md`,
  section 5): the visit printed, the redirect taken, the record kept,
  exit 0; a handed-in account and a confidential client refused; a
  running engine lending the new record at its next grant.
- **The live tier** borrows a Codex login's access token read-only,
  never refreshes it and never writes its file (`../testing.md`, 2.7).

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
- **A handed-in account read again while the engine runs** (jig's
  `protocol/llm.md`, section 10). Default: a newer file is read at the
  next start.
- **The forge as people's sign-in provider,** for a deployment with more
  than one person: routes of temper's on its clients' listener run the
  forge's OAuth flow, read who the person is, and ask jig's clients'
  component to mint a credential for that identity, with a `Mint` from
  temper's server code alone (jig's `protocol/client.md`, section 8;
  jig's `protocol/service.md`, section 2; `../client/README.md`, section 11).
  The forge's OAuth client secret is then temper's, a private file read
  at start. Default: the printed link signs in the deployment's one
  owner.
