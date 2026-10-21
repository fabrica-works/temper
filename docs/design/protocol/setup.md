# The forge's setup

Provisional, 2026-10-16, revised 2026-10-20. How a forge comes to be
ready for temper, in three modes: connected, where a person runs and sets
up the forge and gives temper only what it strictly needs; managed,
where a person runs it and temper sets itself up on it, with a Forgejo
site admin's token or, on GitHub, with an App made from temper's
manifest; and owned, where temper runs Forgejo and its runner, and sets
itself up on them. A fourth, fake, is the demo's: temper runs its own
fake Forgejo, seeded with a demo repository, and sets itself up on it as
on an owned one (8.4). A person chooses a forge's mode and connects it from
temper's client (`../client/forge.md`, the deployment's page). The
engine is the same in every mode: each mode makes the records the one
below reads, kept in jig's secrets' store (credentials.md, the records),
and the engine's forge component reads connected's. Owned mode is
commands any host can arrange; `temper box` is one arrangement, and
wiring only. A box is what temper's first milestone is set up on and
checked by CI on (`../README.md`, section 2), and what QA's agents drive
(`../draft/qa.md`, 4.1). The members are configuration.md's (section 3),
the credentials credentials.md's (section 2), the webhooks forge.md's
(section 5), and the commands and crates README.md's (sections 4 and 5).
A person's walks are `docs/guide/box.md`, `docs/guide/forgejo.md` and
`docs/guide/github.md`. What comes later is section 15; what is open,
section 16; connecting GitHub, section 17.

## 1. In one page

- **Three modes, per forge** (section 2), chosen and connected from
  temper's client. Connected: a person runs and sets up the forge, and
  pastes the bot's two tokens on Forgejo, or an App's id and key on
  GitHub. Managed: a person runs it, and pastes a Forgejo site admin's
  token, or on GitHub makes temper's App from its manifest in their
  browser. Owned: temper runs Forgejo and its runner. A fourth, fake,
  is the default document's and never chosen: temper runs its fake
  Forgejo in place of Forgejo, with no runner and its git a route of its
  own, and sets it up as an owned one (8.4).
- **Each mode is a layer that makes the inputs of the one below.**
  Owned's preparation makes a Forgejo and an admin's token, managed's
  input, and hands the token over in a file; managed's plans make the
  bot, its tokens, its id and the webhooks' secret, connected's inputs.
  The forge component always runs connected: it reads the same records,
  checks the same user, and never holds an admin's token (section 3).
- **The records are the interface.** Each forge's records, under its
  configured `number`, a stable id never reused, are kept in jig's
  secrets' store by the engine's protocol layer alone (credentials.md,
  the records). The one file that passes between the layers is an owned
  forge's handover, `admin_token`, which the engine takes once.
- **The core is step code; the hosts are thin.** The setup's plans are
  skein's `skein-forge-setup` (skein's `forge-setup.md`, sections 4 and
  5), run inside the engine by skein's setup host component (its host,
  6.3): per forge, one writing plan at once and the picker's read beside
  it (5.1). The preparation, Forgejo's settings and the processes' specs
  are machines and pure functions of `temper-forge-setup`, hosted by
  three commands, `temper forge fetch`, `temper forge prepare` and
  `temper forge run` (section 6). No command writes the records.
- **`temper box` is wiring only.** It runs a configuration's steps in
  order (the demo's fakes, fetch, prepare, Forgejo, its runner, `temper
  serve`, the demo's seed), restarts what exits, and stops them in
  reverse order. An owned forge's steps may wait, dormant, until the
  client asks for them over a pipe to `temper serve` (7.1). It names no
  Forgejo request and nothing of setup but its steps. systemd, QA's
  environments or a test arrange the same commands another way (section
  7).
- **`temper init` writes a deployment that starts** into the three
  directories (configuration.md, section 12): the default document, its
  charters, its agent's configuration, the worker's secret and the demo
  model's key; no account and no credential, which are the client's.
  `temper box` runs it first on an empty configuration directory (8.2,
  8.3).
- **A first run is a demo** (8.4): the default document runs a fake LLM
  and a fake forge as the box's steps, and a seed that adds the demo
  model's key and adopts the demo repository through temper's client
  tools, as a person would, before the person's sign-in link is
  printed. Real backends are added from the client beside it: GitHub
  and an owned Forgejo are in the default document, so connecting
  either needs no restart, and the owned one's steps wait until the
  client asks for its setup (8.3, 7.1).
- **Reconcile converges at every start** of a managed or owned Forgejo,
  in the engine: the bot user, made when missing and otherwise checked,
  never edited; its two tokens; an owned forge's person's orgs; and each
  adopted repository's access, the bot a collaborator with write and the
  repository's hook; each made when missing, repaired when changed, and
  left alone otherwise (section 5).
- **Access at adoption.** Adopting a repository of a managed or owned
  Forgejo first gives the bot its access (`Adopt`, 5.4); nothing is
  granted org-wide. The picker offers every repository the site admin
  can see, read by the `Reachable` plan, which writes nothing (5.2).
- **GitHub** (section 17): managed by the manifest flow, through the
  clients' listener's routes, or connected by an App the person pasted;
  both end in the same records. Its reconcile, after each connect,
  repairs only the App's hook's address; its changes reach the engine
  through the App's delivery log, with no webhook secret.
- **Owned is a stock Forgejo.** temper writes its `app.ini`, runs its
  migrations, makes its own admin and the person's account with
  Forgejo's command line, registers the runner offline, dumps before an
  upgrade's migration, and runs only the versions it pins, the
  conformance's, which it fetches itself when asked, checked against
  the digests it pins (section 6). Its directory, once temper stops running
  it, is a Forgejo a person can run themselves, as managed.
- **A box is reactive.** Every change on the forge that a box's work
  waits on reaches the engine as a signed webhook delivery, and becomes a
  hint, read within the keep's `hinted`: pushes, pull requests, issues,
  comments, reviews and each CI job's end. Adoption and reconcile make
  the hooks; an owned Forgejo is allowed to deliver to the engine's
  listener. Polling is unchanged, the fallback for a delivery lost or
  refused (section 10).
- **Loopback.** An owned Forgejo listens on loopback, as temper's
  clients' and webhooks' listeners do; a person on another machine
  reaches them through a tunnel. A forge on another host polls: its
  webhooks need skein's TLS server side (section 15).
- **Not here:** webhooks from another host, the isolation of CI's jobs,
  provisioning at runtime and the forge as people's sign-in provider
  (section 15).

## 2. The modes

| | connected | managed | owned | fake |
|---|---|---|---|---|
| Runs the forge and CI's runner | the person | the person | temper | temper: its fake forge, `temper fake forge`, whose CI answers each push itself; no runner |
| Makes temper's bot, its tokens, its access and the webhooks | the person | temper: on Forgejo with a site admin's token; on GitHub, an App from temper's manifest | temper, with an admin it makes | temper, with the admin the fake is seeded with |
| temper is given, on Forgejo | the bot's two tokens, pasted; optionally a webhooks' secret | a site admin's token, pasted | the Forgejo and runner binaries, or `"fetched"` for temper's (6.6); an address on loopback; the person's login, email and orgs | an address on loopback; the seed's owner and repositories |
| temper is given, on GitHub | an App's id and its private key, pasted | the person's steps in their browser, from a link temper prints | none: GitHub is never owned | none: the fake is a Forgejo |

- **Fake is owned's layers,** with temper's fake forge in place of
  Forgejo and its runner (8.4). What this document and the others say
  of an owned forge's handover, directory, reconcile, records, adoption
  and picker holds of a fake one; what they say of Forgejo's or the
  runner's binaries, `app.ini`, the preparation's command line and the
  person does not. A fake forge
  keeps nothing across its starts but its directory's handover: each
  start serves the seed afresh, and reconcile makes the bot again
  (8.4).
- **Per forge.** A deployment may mix them: an owned Forgejo beside a
  connected one, or beside GitHub, or the demo's fake beside a person's
  real forges. Each owned or fake forge has a directory and an address
  of its own.
- **A mode is no record** (credentials.md, the records): a forge is
  owned or fake when its `setup` entry says so (section 8); managed when
  it is neither and its `AdminToken` record is kept; connected otherwise. On
  GitHub, managed and connected end in the same records, and differ only
  in how the App came to be (section 17).
- **A mode changes:**
  - owned to managed, by the configuration and a restart: the forge's
    `setup` entry says `"managed"`, temper stops running Forgejo, and
    its directory's `forgejo/` is a stock work path a person runs; the
    `AdminToken` record, taken over (6.5), stays valid;
  - managed to connected, from the client, "Forget the admin's token":
    the `AdminToken` record is erased, no reconcile runs for the forge,
    and its bot, tokens, id and hooks stay as reconcile last kept them;
    adopting a repository then grants nothing, and the person makes the
    bot a collaborator (section 4);
  - connected to managed, from the client, "Connect as its admin": a
    site admin's token is pasted, and reconcile runs with it, taking
    over the connected user as its bot when its full name carries
    `temper:bot` (5.5).

## 3. The layers and their files

The three modes and their layering (each mode makes the inputs of the
one below, and the service always runs connected) are held by skein's
`forge-setup.md`, section 3; fake is owned's layer with another forge in
it (section 2). temper's layers, and what passes between them:

```
owned      temper forge fetch        ──► the Forgejo and runner binaries, in the cache (6.6)
           temper forge prepare      ──► the forge's work path, the runner's, person_password,
                                         admin_token (the handover)
           temper forge run          ──► forgejo web, forgejo-runner daemon (each exec'd)
fake       temper forge prepare      ──► the forge's directory, admin_token (the handover)
           temper fake forge         ──► the fake, seeded with temper-admin and that token (8.4)
managed    the engine's setup host   ──► the forge's records (below); takes the handover once
connected  the forge component       ◄── its access, built from the records
```

- **One directory per owned or fake forge,** named by its setup (section 8):
  private, `0o700`, the user's own and not reached through a link, as a
  worker's root is (configuration.md, section 4). `temper forge
  prepare` makes it so when it is absent; a directory that is not so
  refuses the command, naming it. A managed or connected forge has
  none. A fake forge's holds `admin_token` alone, which `temper fake
  forge` reads too (8.4). An owned forge's files:

  | File | Made by | Read by | Holds |
  |---|---|---|---|
  | `forgejo/` | prepare | Forgejo, through `run` | Forgejo's work path: `app.ini`, its database, repositories and logs |
  | `runner/` | prepare | the runner, through `run` | the runner's configuration, its registration, its jobs' directories, `secret` |
  | `backups/` | prepare | the person | one dump before each upgrade's migration |
  | `state` | prepare | prepare, `run` | owned's record: the versions prepared, whether the first preparation ended, Forgejo's own secrets |
  | `admin_token` | prepare, when missing | the engine's setup host, once, at the start that finds it | the handover: temper's admin's token, kept as the forge's `AdminToken` record by `TakeOver`, after whose `Done` the file is removed (6.5) |
  | `person_password` | prepare | the person | the person's first password on the owned Forgejo |

- **What the preparation leaves for the engine** is `admin_token`
  alone: the token's text, nothing else, private and written whole;
  made at the first preparation and at each later one that finds it
  missing, which it is once the engine has taken it, and left as it is
  when present (6.3).
- **Fetched binaries are the cache's,** not the forge's directory's:
  `<cache>/forgejo/VERSION/forgejo` and
  `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner`, shared by
  every owned forge whose binaries are `"fetched"`, written by `temper
  forge fetch` alone and fetched again at will (6.6; configuration.md,
  section 12).
- **No other file is read.** temper reads no `api_token`, `git_token`,
  `user` or `webhook_secret` file: a forge's directory that holds them,
  from a temper before the records, keeps them unread, and nothing is
  converted. Every record is written by a plan, `User` as its 8 bytes
  within a version-1 record (credentials.md, the records). An owned
  forge's records are made afresh at the start that takes its handover
  (6.5); a managed forge whose records are missing is not connected, and
  is connected again from the client (5.5).
- **Written whole.** Each file is written beside its name and renamed
  over it, private, so a reader never sees half a token.
- **The handover is read** as configuration.md, section 6 reads a
  secret's file, within its bound: it is the one file of `setup` that
  `temper serve` reads.
- **The records, by mode** (credentials.md, the records):

  | Forge | Records |
  |---|---|
  | Forgejo, connected | `ApiToken`, `GitToken`, `User`, `Origin`; `WebhookSecret` when the person gave one |
  | Forgejo, managed, owned or fake | those, and `AdminToken`; `WebhookSecret` drawn by reconcile when the webhooks' listener is configured |
  | GitHub, managed or connected | `AppId`, `AppSlug`, `AppKey`, `User` (the App's bot user), `Accepted`, `Origin`; no `WebhookSecret` |

- **Written by the setup host's plans alone,** each write kept before
  the plan goes on (5.1). A value a person pastes is kept by jig until
  its run ends (credentials.md, pasted values), and is no record.
- **Records made for another forge,** an `Origin` naming another kind
  or address, and unreadable ones, leave that forge not connected,
  named on the deployment's page; nothing is erased, and connecting
  again overwrites them (credentials.md, the records).
- **Nothing passes between the layers** in an argument or the
  environment but the configuration's path and the forge's number.

## 4. Connected

- **What temper strictly needs** of a connected Forgejo: the bot's API
  token and its git token, and, for webhooks, their secret; the bot's
  id is read with the tokens (credentials.md, sections 4 and 5). The
  scopes (skein's `forge-setup.md`, section 8): `read:user`,
  `write:issue` and `write:repository` for the API token;
  `write:repository` and `read:user` for the git token, the second so
  that the setup's plans can ask whose it is (skein's `forge-setup.md`,
  4.3, step 3, and 4.6).
- **What the person does:** a Forgejo user for temper, not a site admin;
  its two tokens; its write access to each repository temper will adopt,
  as a collaborator or through a team; optionally, each repository's or
  org's webhook to temper's listener, `http://<listener>/hooks/<number>`,
  with the secret and the events of skein's `forge.md`, section 7, and
  Forgejo's `[webhook] ALLOWED_HOST_LIST` letting it reach the listener;
  CI, as each repository has it.
- **Pasted from the client, together** (`Connect` with `ForgejoTokens`;
  `../client/forge.md`, the deployment's page): both tokens, and the
  secret when given, one value, kept by jig until its run ends
  (credentials.md, pasted values). The setup host's `Check { forge }`
  (skein's `forge-setup.md`, 4.6) finds that both answer as one user,
  not a site admin, and writes that user, and the routing keeps the
  tokens and the secret as the forge's records; the page then says
  "Connected as LOGIN (ID)". Tokens of two users stop it `OtherUser`, a
  site admin's `BotIsAdmin`, a refused token `Refused`, each named, and
  the forge stays as it was.
- **Replaced** from the client (`Replace`): a new token, checked to
  answer as the kept user, kept before the old is erased.
- **Nothing more is required** (`../domain/forge.md`, section 4): polling
  stands in for webhooks, and a project's checks for CI.
- **A connected GitHub** is an App the person made, pasted (17.3).

## 5. Managed: reconcile

Reconcile is skein's plan, run by the engine's setup host at each start
of a managed or owned Forgejo, at a managed connect and at an owned
forge's set up from the client (5.1). No command runs
it, and nothing but the setup host writes what it makes.

### 5.1 The setup host in the engine

- **skein's host component** (skein's `forge-setup.md`, its host, 6.3),
  a child of the engine's protocol layer beside the forge component; the
  routing carries what passes between it, the root and the records
  (forge.md, the setup's host). It holds an admin's token and a GitHub
  App's key only while a plan that asked for them runs, and drops them
  at its end.
- **Per forge, one writing plan at once, and one `Reachable` beside
  it,** since the picker's read writes nothing. A second of either kind
  is refused "busy", never queued, and so is one past `setup.runs`,
  which counts both kinds.
- **Its needs:** a write is a `Keep` of the forge's record, answered
  `Written` only once `Kept`, so a browser is answered and a connect
  sent only on what a restart would find; a read is served from the
  run's pasted value first, then from the forge's record. It draws its
  random bytes from the engine's source.
- **States,** for GitHub's browser steps, are its own, in memory: at
  most `setup.states`, each living `setup.state_life_ms`, an hour at
  most; a restart forgets them (17.2).
- **At each start,** once the records are loaded: an owned or fake
  forge's handover, when its file exists, by `TakeOver` (6.5); then
  `Reconcile` for each managed, owned and fake Forgejo; and GitHub's
  `Reconcile` after each
  GitHub forge connects (17.5). The deployment's page shows a forge
  "Connecting" until its plan is done and it connects.
- **At the box's `started`,** for an owned forge set up from the client
  while the engine serves (7.1): its handover by `TakeOver`, then its
  `Reconcile`, then its connect, as a pasted admin's token is taken
  (`../draft/github.md`, 12.2). The row shows "Connecting" meanwhile;
  a stop leaves the forge not connected, named, as a start's does.
- **A run's end:** `Done`, or `Lacking` (done, with what the App lacks
  named), keeps its writes, then connects the forge after a plan that
  connects; `Stop` names its failure and its step, with no secret, and
  leaves the forge as it was. A start's reconcile that stops leaves that
  forge not connected, named, until the next start or a managed connect;
  the engine and every other forge run on.

### 5.2 The setup host's plans

| Plan | Forge | When | skein's `forge-setup.md` |
|---|---|---|---|
| `Reconcile { forge, adopted }` | Forgejo, managed, owned or fake | each start; a managed connect; the box's `started` (5.1) | section 4 |
| `Adopt { forge, repository }` | Forgejo, managed, owned or fake | an adoption, before its reads | 4.5 |
| `Check { forge }` | Forgejo, connected | a connect; a replacement | 4.6 |
| `TakeOver { forge, admin_token }` | Forgejo, owned or fake | a start that finds the handover; an owned forge's `started` from the box (5.1) | 4.7 |
| `Reachable { forge, installation?, page }` | every kind | the picker | 4.8 |
| `Manifest { forge, target }` | GitHub | Connect, managed | 5.1 |
| `Convert { forge, code, state }` | GitHub | the redirect back | 5.2 |
| `Install { forge }` | GitHub | Install on another account | 5.4 |
| `Accept { forge, installation, state }` | GitHub | the setup route's visit | 5.4 |
| `Reconcile { forge }` | GitHub | after each connect | 5.6 |
| `Paste { forge, app }` | GitHub | Connect with a pasted App; Replace | 5.8 |

- **What each does, and when the routing runs it,** is forge.md's (the
  setup's host); each plan's steps, its bounds and what it refuses are
  skein's.
- **`Reachable`** serves every kind, pages counted from 1, at most
  `reachable` rows, which temper gives from `picker.page`
  (configuration.md, section 7), and ends `Done(Reachable { rows, more
  })`, `more` when the page is full; it writes nothing. On GitHub it
  reads one accepted installation's repositories, `installation` its
  id, on a token minted for the page and dropped; an id not in
  `Accepted` stops it `Installation`, `Missing`. On a connected Forgejo
  it reads what the API token can push to; an `installation` given
  there stops it `Installation`, `Unexpected`. On a managed, owned or
  fake Forgejo it reads every repository the admin can see, by id ascending,
  on the admin's token, which stays the setup host's, and is refused
  `Missing` or `NotAdmin` as reconcile's first step is. Forgetting that
  token makes the forge connected, and its picker with it.
- **Outcomes:** `Done`; `Manifest { state, page }`, the state and the
  manifest page's bytes, which temper holds with the state; `Link`, the
  install link; `Lacking`, done, with each permission or event the App
  lacks named.

### 5.3 Reconcile's given values, and its answers

- **The plan is skein's.** Its six steps, in this order (1 the admin; 2
  the bot, marked `<word>:bot`; 3 the bot's two tokens; 4 the bot's id;
  5 each org, when owned; 6 each adopted repository's access, the bot a
  collaborator and the repository's hook), each read first and written
  only where the forge differs; its wait for the forge; its bounds; and
  what it refuses are held by skein's `forge-setup.md`, section 4. It
  does no io and reads no configuration: it is given values, says each
  request, and the setup host runs it.
- **Its given values, from temper's configuration:**

  | Reconcile's value | temper's |
  |---|---|
  | the forge's number | the forge's `number` (configuration.md, section 3) |
  | the bot's login and email | the setup's `bot.login` and `bot.email` (section 8); at a connect as its admin of a forge connected before, the connected user's login (5.5) |
  | the tokens' names | `temper-api` for the API's, `temper-git` for git's |
  | the orgs, and their owner | `mode.owned.person.orgs` and `mode.owned.person.login` when owned (6.5); none when managed or fake |
  | the repositories | the forge's adopted repositories, the root's, restored before the start's reconcile |
  | the name a failure gives the admin | `temper-admin` |
  | the listener's address | `application.webhooks.listener`'s, when configured; none otherwise. Each hook's URL, `http://<address>/hooks/<number>`, is derived from it (skein's `forge-setup.md`, 4.1) |
  | the word | `temper`, so the bot's mark is `temper:bot` |
  | its limits | `orgs` from `setup.orgs`, `repositories` from `forge.client.repositories`, `reachable` from `picker.page`, and `deadline`, `backoff` and `backoff_max` from `setup.deadline_ms`, `setup.backoff_ms` and `setup.backoff_max_ms` (configuration.md, section 7) |

  The API's address is the forge's `api` (configuration.md, section 3).
  temper-forge-setup maps the setup onto these values and runs the
  kit's check on them, at each start and for every command (section
  11); a value the check refuses comes back as its `Place` (skein's
  `forge-setup.md`, 6.1), and is named by the path this table gives for
  it.
- **What the setup host answers, as needs:**

  | Reconcile asks for | The setup host answers |
  |---|---|
  | the admin's token | at a managed connect, the pasted value; otherwise the `AdminToken` record |
  | the API and git tokens, the bot's id | the `ApiToken`, `GitToken` and `User` records, or none when absent |
  | the webhooks' secret | the `WebhookSecret` record, or none, and reconcile then draws one |

  Each answer is a record or a pasted value, never a file: a forge's
  `api_token`, `git_token`, `user` or `webhook_secret` file, from a
  temper before the records, is not read, and reconcile makes what is
  missing afresh (section 3). Each item reconcile writes is kept as the
  forge's record; a kept `WebhookSecret` is never drawn again.

### 5.4 Access at adoption

- **`Adopt { forge, repository }`** runs before an adoption's reads on a
  managed, owned or fake Forgejo (skein's `forge-setup.md`, 4.5): the bot made
  a collaborator with write, and the repository's hook made when the
  listener is configured, with the forge's secret. Stopped, the adoption
  is refused, named (`../domain/forge.md`, access at adoption). A
  `403` here, at reconcile's step 6 or at the picker's read, is the
  admin's token without `write:repository` (skein's 4.3, step 1): it
  is shown as that scope missing, named with its step.
- **At each start,** reconcile's step 6 checks each adopted
  repository's access, and repairs what changed.
- **Nothing org-wide:** the bot reaches the repositories adopted, a
  person's own among them, and no other.
- **A connected Forgejo, and a managed one forgotten,** grant nothing at
  adoption: the person makes the bot a collaborator, and the repository
  polls unless the person adds its hook (section 4).

### 5.5 Connecting as its admin, and forgetting

- **"Connect as its admin"** (`Connect` with `ForgejoAdmin`;
  `../client/forge.md`, the deployment's page), on a Forgejo not owned
  and not connecting, whose `setup` entry is managed (section 8): the
  site admin's token, with the preparation's scopes (6.3), pasted, kept
  by jig until its run ends, and
  `Reconcile` run with it. Done, the token is kept as the forge's
  `AdminToken` record beside reconcile's writes, and the forge connects,
  managed.
- **Its bot:** on a forge with no records, the setup's `bot.login`, made
  when missing; on a forge connected before, the connected user's login,
  taken over when its full name carries `temper:bot`, or the run stops
  `Unmarked`, named, and the forge stays connected.
- **A Forgejo with no `setup` entry** refuses it, naming
  `application.setup` (section 16).
- **"Forget the admin's token"** (`Forget`): the `AdminToken` record is
  erased, and the forge is connected, its bot, tokens, id and hooks
  kept; no reconcile runs for it after.

### 5.6 What holds of every plan on Forgejo

- **Plaintext to loopback alone:** a managed or owned forge's
  `transport` is plaintext, and one whose transport is TLS is refused at
  start, naming it (section 15).
- **The bot's email** is the setup's `bot.email` when reconcile makes
  the bot, and stays as it is after: a change to `bot.email` reaches
  only a bot made after it, and an existing bot's email is the
  person's to change, on Forgejo. temper reads no mail of the bot's,
  and its commits name the identity's `author` (configuration.md,
  section 3), not this email.
- **The scopes** of the tokens are section 4's.
- **What it cannot set:** Forgejo's settings, which its API does not
  change (skein's `forge-setup.md`, section 4): the webhooks'
  `ALLOWED_HOST_LIST`, and whether Actions are enabled. A managed forge
  whose host list refuses temper's listener delivers no webhook, and
  polling keeps up (section 10).

## 6. Owned: prepare and run

### 6.1 Its inputs and its binaries

- **Inputs:** the Forgejo binary, and the runner's, or no runner, each
  an absolute path or `"fetched"`, temper's pinned build in the cache
  (6.6); the address Forgejo listens on, a socket address on loopback;
  the person's login, email and orgs; the runner's labels, capacity and
  `PATH`; the forge's directory.
- **The binaries are checked** by every command that runs them: the
  first line of each one's version equal to temper's pin, Forgejo's and
  the runner's (`skein-forge`'s `VERSION` and `RUNNER_VERSION`, skein's
  `forge.md`, section 4.3), the versions skein's conformance checks
  (`fake-forge.md`, section 7). Another binary refuses the command,
  naming its path and what it printed. A fetched binary is checked so
  too, after its digest at its fetch; temper downloads nothing but by
  `temper forge fetch` (6.6).
- **One pin,** `skein-forge`'s, read by owned mode, the throwaway and
  the conformance alike, so the Forgejo temper runs is the Forgejo whose
  facts it checked (13.3): the versions, and beside them each fetched
  binary's SHA-256 for each architecture temper fetches for (6.6).

### 6.2 Forgejo's settings

`app.ini`, written by every preparation from the setup and `state`,
private, never edited by hand:

- `RUN_USER` the process's user, `WORK_PATH` the work path;
- `[server]`: `HTTP_ADDR` and `HTTP_PORT` from the address, `ROOT_URL`
  `http://<address>/`, `DISABLE_SSH`, `OFFLINE_MODE`;
- `[database]`: SQLite in the work path;
- `[repository]`: its root in the work path, `DEFAULT_BRANCH` `main`;
- `[security]`: `INSTALL_LOCK`, `SECRET_KEY` and `INTERNAL_TOKEN`;
  `[oauth2]`: `JWT_SECRET`; each drawn once, at the first preparation,
  and kept in `state`;
- `[service]`: `DISABLE_REGISTRATION`; `[mailer]`: off;
- `[webhook]`: `ALLOWED_HOST_LIST` the webhooks' listener's IP address,
  when one is configured, and Forgejo's default otherwise, which refuses
  loopback;
- `[actions]`: `ENABLED` when the setup has a runner;
- `[log]`: files in the work path;
- `[api]`: Forgejo's defaults, which the forge component reads at its
  start (skein's `forge-connection.md`, section 3).

### 6.3 The preparation

`temper forge prepare [DIRECTORIES] FORGE`, before Forgejo runs. It takes
Forgejo's lock (6.4), and the runner's too when it writes the runner's
files: at a first preparation, and for a runner older than its pin
(item 3) or added (item 4); checks the binaries; and decides by
`state`:

1. **The first preparation:** the layout of section 3; `app.ini`;
   `forgejo migrate`; temper's admin, `temper-admin`, made as a site
   admin with a password Forgejo draws and nobody keeps, and its token
   made by `admin user generate-access-token` into `admin_token`, the
   handover (section 3), with the scopes `write:admin`, `read:user`
   and `write:repository`, the three skein's `forge-setup.md` names
   (4.3, step 1: the users, their tokens and the orgs; `GET /user`; and
   the repositories' routes of `Adopt`, reconcile's step 6 and the
   picker's read), and no other, under a name that
   ends in a random suffix, since Forgejo refuses a second token of one
   name; the
   person's account, a site admin with a password Forgejo draws into
   `person_password`, to change at their first sign-in; the runner's
   secret drawn into `runner/secret` (40 hexadecimal digits, no line
   feed), registered as a global runner by `forgejo-cli actions register
   --secret-file`, and the runner's connection written from it into the
   runner's configuration (`server.connections`), its token in a private
   file the connection names (`token_url: file:`); the work path's
   repositories directory, which `forgejo dump` needs. `state` records
   the first preparation's start before anything else, and its end last.
   A first preparation that `state` says began and did not end is cut
   short: the next one removes what it left (the work path, the runner's
   directory, `admin_token`, `person_password`) and begins again from an
   empty layout, since Forgejo refuses a user made twice and never shows
   a drawn password again. A forge's directory that holds Forgejo's files
   with no `state` is refused, naming it: the preparation removes only
   what a first preparation it recorded left.
2. **A later preparation at the same versions, with a runner exactly
   when `state` records one:** `app.ini` written again, so a change of
   the setup reaches Forgejo; `admin_token` made again when missing,
   which it is once the engine has taken it (6.5), under a fresh suffix,
   for temper's admin, which Forgejo's command line cannot make twice:
   an admin a person deleted fails the preparation, naming it; nothing
   else. It takes Forgejo's lock alone, so a box restarts a Forgejo,
   after its preparation, while the runner runs.
3. **An upgrade:** a recorded Forgejo older than the pin is dumped
   (`forgejo dump`) into `backups/forgejo-<old>-to-<new>.zip`, written as
   `backups/dump.partial.zip` (Forgejo adds `.zip` to a name without it)
   and renamed, never overwritten (a dump already there is not made
   again), then migrated, then recorded. A recorded runner older than its
   pin has its files prepared again, under the runner's lock too, with no
   dump, since the runner keeps no data. A recorded version newer than
   its pin, Forgejo's or the runner's, refuses: Forgejo's migrations only
   go forward.
4. **A runner added or removed:** the setup and `state` disagree on
   whether there is one, and `app.ini`'s `[actions]` follows the setup
   (6.2). A runner added is prepared under the runner's lock too, as a
   first preparation prepares it: its secret drawn into `runner/secret`
   when that file is missing, and kept otherwise; registered; its
   connection written. A runner removed is recorded as none, and its
   files and its registration are left, unused, so that a runner added
   again registers as the same one. A runner's recorded version is
   compared with its pin only while the setup has one. Either may come
   with an upgrade, in one preparation.

- **Forgejo's command line only,** on the work path, never its web: under
  the lock no Forgejo runs, so nothing writes its database beside the
  preparation.
- **No secret in an argument.** A password is drawn by Forgejo (the user
  creation's random password) and read from its output; the runner's
  secret goes by file (`--secret-file`). The runner's `create-runner-file`
  takes its secret only as an argument, so the preparation never runs
  it: it writes the runner's connection itself, in the runner's
  configuration, with its token in a file (13.3).
- **Its commands run on skein-io's processes,** as the box's children
  do: each spawned with its arguments and the spec's environment, its
  output read through pipes within a bound, its exit status awaited.
- **Interrupted** by a termination signal while a command runs, it
  kills the command's process group and exits 1, printing `forge <n>:
  interrupted`. `state` is not marked ended, so the next preparation
  begins again from an empty layout, for a first one, or repeats, for a
  later one or an upgrade.
- **Its commands' output,** kept for a failure's message, is masked of
  every secret it holds, as the throwaway Forgejo masks its own.

### 6.4 Running

`temper forge run [DIRECTORIES] FORGE forgejo|runner`:

- **Checks** the binary, and that `state` records a preparation at the
  pinned versions, with a runner exactly when the setup has one;
  refuses otherwise, naming what to run: `temper forge prepare` for a
  forge not prepared, a preparation that did not end, an older version,
  or a runner added or removed (6.3); no command of this temper for a
  newer version, or one it cannot read.
- **Takes the lock,** an exclusive lock, taken without waiting, on a
  file of the forge's directory, Forgejo's or the runner's, and hands it
  across the exec as the process's standard input, so the process holds
  it for its life. Two Forgejos never share a work path, and no
  preparation runs under one: the preparation takes Forgejo's lock, and
  the runner's too when it writes the runner's files (6.3), and refuses,
  naming the forge, while one it needs is held.
- **Replaces itself** (`execve`) by `forgejo web` or `forgejo-runner
  daemon`, with the spec's arguments and the spec's environment alone:
  cleared, then `HOME` in the forge's directory, `PATH`, and git's
  configuration isolated, as the throwaway Forgejo's is. The process a
  host supervises is then Forgejo's own, with its signals and its exit
  status.
- **Outside skein's shell, in one module.** skein's shell spawns but
  never execs, and its locks never pass to a child (skein's
  `programming-model.md`, 2.1; skein's `io.md`, section 5). So the lock and
  the exec are std's, `File::try_lock` and `CommandExt::exec`, with no
  `unsafe`, in one module of `temper-forge-setup-shell`, which the
  preparation's lock uses too; the preparation's commands are not in
  it, and run on skein-io (6.3). It is the one place temper locks a file
  or uses `std::process` outside skein: a boundary test scans the
  production crates' sources (`crates/*/src`, test files aside) and
  fails the gate when either appears outside that module, but
  `std::process::ExitCode` in the binary's `main`. Test crates
  (`testing/`, `tools/`) are not scanned. It moves onto skein when
  skein's shell offers both (section 15).
- **The runner's spec:** its configuration file (its connection,
  capacity, labels, timeouts, its jobs' directory, no cache), its `PATH`
  from the setup. Its jobs run as the user that runs it (section 15).

### 6.5 Then the engine's start, and the person

- **The handover,** at `temper serve`'s start, or at the box's `started`
  for a forge set up from the client (7.1): the setup host finds
  `admin_token`, reads it once, and runs `TakeOver { forge, admin_token
  }` (skein's `forge-setup.md`, 4.7): the token checked as a site
  admin's, kept as the forge's `AdminToken` record, and temper's admin's
  other tokens deleted, so no older one stays valid. The file is
  removed after its `Done`, never before; a `TakeOver` that stops leaves
  it, and the forge not connected, named. A start with no file uses the
  record.
- **Then reconcile runs as managed's** (section 5), and makes each of
  the person's orgs that is missing, owned by the person (`POST
  /api/v1/admin/users/{person}/orgs`).
- **A forge not prepared,** with neither a handover nor an `AdminToken`
  record, is not connected, and is set up from the client
  (`../client/forge.md`, 5.4):
  - *under the box,* an owned forge whose setup says `"start": "asked"`
    is dormant (7.1), and the deployment's page offers "Set up a
    Forgejo here". The engine's routing writes `start FORGE` on the
    box's pipe; at `started FORGE` the setup host takes the handover,
    reconciles and connects (5.1), and at `failed FORGE WHY` the page
    names why and offers it again;
  - *without the box* (7.4), `temper serve` has no pipe, and the page
    names the commands to run: `temper forge fetch`, `temper forge
    prepare`, `temper forge run FORGE forgejo`, `temper forge run FORGE
    runner`, then a restart.
- **The person** signs in on Forgejo's web as a site admin with
  `person_password`, and changes it, since Forgejo's API refuses them
  until they do (13.3); makes repositories in their orgs, with Actions,
  and adopts them through temper's client. temper's admin is temper's; a
  person who deletes it makes the next start's reconcile and the next
  preparation stop, naming it.
- **The forge's members** are connected's, written out: `api` the
  address, `transport` plaintext, `git` `http://<address>` (section 8).

### 6.6 Fetching

`temper forge fetch [DIRECTORIES] FORGE`, before an owned forge's
preparation, for a forge whose `forgejo` or `runner.binary` is
`"fetched"` (section 8): a once step of the box (7.1), and a command of
its own for any other host.

- **What it fetches,** each binary its setup gives as `"fetched"`, at
  temper's pins (6.1), for the machine's architecture, `ARCH`:

  | Binary | From | Into |
  |---|---|---|
  | Forgejo | `https://codeberg.org/forgejo/forgejo/releases/download/vVERSION/forgejo-VERSION-linux-ARCH` | `<cache>/forgejo/VERSION/forgejo` |
  | the runner | `https://code.forgejo.org/forgejo/runner/releases/download/vRUNNER_VERSION/forgejo-runner-RUNNER_VERSION-linux-ARCH` | `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner` |

  `ARCH` is `amd64` or `arm64`; another machine refuses, naming its
  architecture, and its binaries are given by path.
- **One request, one answer.** Each URL answers `200` with the binary,
  never a redirect: skein-http follows none, and any other status
  refuses, naming the URL and the status.
- **A mirror,** the owned entry's `mirror`, absent by default (section
  8): an `https` origin for each binary, in place of codeberg.org's and
  code.forgejo.org's, its paths and digests the pins' still. A host that
  keeps its own copies names it, and the process tier its local file
  server (13.4).
- **TLS with the machine's roots,** read at the command's start as
  skein's shell reads them (skein's `shell.md`, 6.2): the bundle
  `SSL_CERT_FILE` names, or the first of the distributions' bundles that
  exists. skein-tls embeds none, so a machine with neither refuses the
  command.
- **The digests are pinned,** one SHA-256 per binary and architecture,
  in `skein-forge` beside `VERSION` and `RUNNER_VERSION` (skein's
  `forge.md`, 4.3), and required: a fetched binary is the build the
  conformance checked, byte for byte.
- **Streamed, never held:** the body's chunks are asked for one at a
  time, as the file takes them; each is hashed into the digest, which
  is incremental, and written at its offset into `<file>.part`, private,
  within `setup.fetch_bytes` (configuration.md, section 7). At the
  body's end, a digest equal to the pin makes the part file `0o755` and
  renames it over its name.
- **A wrong digest** removes the part file and refuses, naming the URL,
  the pinned digest and the digest it got. Nothing is left under the
  binary's name.
- **A file already there is hashed,** never fetched again: its pin's
  digest is nothing to do; another is refused, naming its path, and
  never overwritten, so a binary a person put there stays theirs to
  remove.
- **A part file an interrupted fetch left** is written again from its
  start.
- **Then the preparation checks its version** as every command that
  runs it does (6.1); a digest is no version.
- **Its code:** the fetch's machine in `temper-forge-setup`, its
  requests, chunks, writes and digests its inputs and outputs, and its
  host in `temper-forge-setup-shell`, on skein-io, skein-http and
  skein-tls (section 11). It holds no credential and writes no record
  (section 9).

## 7. The box: wiring only

### 7.1 What it runs

`temper box [DIRECTORIES]` runs the configuration's steps, which
`temper-fake`'s `steps` gives (8.4), each a command of the `temper`
binary, as a child process given the box's three directories as flags,
`--config C --data D --cache K`, each absolute (configuration.md,
section 12), written `DIRS` below:

```
temper box
├── temper fake llm DIRS              long, with the demo (8.4)
├── temper forge prepare DIRS 1       once, per fake forge: its handover
├── temper fake forge DIRS 1          long, per fake forge (8.4)
├── temper forge fetch DIRS 3         once, per owned forge whose binaries are fetched (6.6)
├── temper forge prepare DIRS 3       once, per owned forge
├── forgejo web                       long: temper forge run DIRS 3 forgejo
├── forgejo-runner daemon             long: temper forge run DIRS 3 runner
│                                     these four, the forge's group: here, or asked (below)
├── temper serve DIRS --box           long: its own tree (README.md, section 4); its setup
│                                     host takes each handover and reconciles (section 5);
│                                     fds 3 and 4 its pipe to the box (below)
└── temper fake seed DIRS             link, with the demo: serve's first line on its stdin (8.4)
```

- **An empty configuration directory,** absent or with no entry, is
  written first by `temper init` (8.2), in the box's own process, before
  the document is read; the box says so on stderr, `box: wrote the
  configuration, <path>`, and goes on to its steps. Otherwise the box
  never writes the document: a configuration directory with entries but
  no document is refused as every command refuses it.
- **An owned forge's group,** its fetch, its preparation, its Forgejo
  and its runner, runs in its place in the order at every start of the
  box when its setup has no `start`. With `"start": "asked"` (section 8)
  the box knows the group's steps and starts none of them until
  `temper serve` asks on the pipe, or until the box's record
  `<data>/box/started/FORGE` exists: one empty file per forge, written
  once the group's preparation has exited 0, so that a later box runs
  the group in its place, as an unasked one, before `temper serve`.
  The record is written beneath the data directory, `box` and
  `box/started` made as needed, mode 0600. When it cannot be written,
  the box's stderr names why, the group answers
  `failed FORGE record not written` and is dormant again. At its start
  the box looks for each asked group's record; one it cannot look at
  is named on its stderr, and its group stays dormant.
- **The pipe,** two of them, made again at each spawn of `temper
  serve`: its fd 3, serve to box, and its fd 4, box to serve. `--box`
  tells `temper serve` to adopt them; without it, as under every other
  arrangement (7.4), serve has no pipe. One line each, ending in a line
  feed, `FORGE` the forge's number:

  | Line | From | Says |
  |---|---|---|
  | `start FORGE` | serve, at the client's ask (6.5) | start the forge's group |
  | `step FORGE STEP` | the box | the group's step `fetch`, `prepare` or `run` is starting, for the client's row (`../client/forge.md`, 5.4) |
  | `started FORGE` | the box | the group's long steps are started, and its record written |
  | `failed FORGE WHY` | the box | the group stopped: `WHY` names the step by its name, as the box's stderr prefixes it (7.2), such as `prepare 3 exited 1`: `<step> exited <code>`, `<step> killed by signal <n>`, `<step> did not start`, or `record not written`, cut to fit the line's bound, and holds no secret; a step's own lines are on the box's stderr (7.2) |

  One ask at a time: a `start` for another forge while a group starts,
  or while a failed group's steps close, is answered `failed FORGE busy`;
  one for the forge whose group is starting, by that group's own lines;
  one for a forge with no group, `failed FORGE not asked`; one for a
  started group, `started FORGE`; one while the box stops,
  `failed FORGE stopping`. An
  answer to a `temper serve` that has exited is dropped: its restart
  finds the handover at its start (6.5). Each line is at most
  `box.line_bytes` (configuration.md, section 7), which both sides
  read within; another line, or a longer one, is named on its reader's
  stderr and ignored.
- **A group's failure** before `started` answers `failed`, and the group
  is dormant again, to be asked again; the box runs on. Once started,
  its long steps are restarted, and given up on, as 7.2 says.
- **The default document's steps** are the demo's: the fake LLM, the
  fake forge's preparation and the fake forge, `temper serve` and the
  seed; and forge 3's group, dormant until the client asks for it
  (8.3).

### 7.2 What it does

- **In order:** each step starts once the one before it has started, when
  that one is long, or has exited 0, when it is once or link. A link
  step starts once the long step before it has written its first line
  to its stdout.
- **A link step** is given that line whole on its stdin, which is then
  closed; it is the link step's alone, and the box's stdout shows the
  long step's lines after it. A long step that exits before writing a
  line hands its restart's first line to the link step. A link step
  runs once per box: the long step's restarts never run it again.
- **Restarts** a long step that exits, after a backoff doubling from
  `box.backoff_ms` to `box.backoff_max_ms` (configuration.md, section 7),
  first running again the steps that are once just before it: a Forgejo is
  restarted after its preparation, so a change of the setup reaches its
  `app.ini`. `temper serve` restarts on its own, with a pipe made afresh,
  and its start takes a handover the preparation left and reconciles each
  managed and owned forge. A token revoked under a running engine stops
  that forge alone, named, and not the engine (forge.md, a forge stopped);
  the engine's next start makes it again. A fake forge's restart is such a
  revoke: it serves its seed afresh, without the bot, so that forge stops
  alone until the engine's next start takes the handover its preparation
  drew again (8.4).
- **Gives up,** exit 1, naming the step, when a once or link step fails,
  but a dormant group's before its `started`, which answers `failed`
  (7.1), or a long one exits more than `box.restarts` times within
  `box.window_ms`. The backoff is counted from a step's exits within the
  window, so it starts again from `box.backoff_ms` once a step has run a
  window without exiting.
- **Stops** at a first termination signal: the long steps in reverse
  order, each by a signal, `box.grace_ms`, a second signal, which turns
  `temper serve`'s drain into an abort (README.md, section 4),
  `box.grace_ms` again, then a close. The machine knows a step only as
  once, long or link, so every long step, and a link step still
  running, is sent both; a dormant group's steps run nothing, and are
  sent nothing. A second termination
  signal closes them all. It exits 0 after a stop it was asked for.
- **Its streams:** `temper serve`'s stdout, the sign-in links, is the
  box's stdout, as is, but for its first line when a link step follows
  it, which is that step's (8.4). Every other line a step writes, to its stdout or
  its stderr, goes to the box's stderr prefixed `<step>: `; the box's own
  lines are prefixed `box: `, such as `box: <step> gave up after <n>
  restarts`. Each step gets the box's environment; `temper forge run`
  clears it for Forgejo (6.4). Forgejo logs to its own files.
- **Its children** are skein-io's (skein's `io.md`, section 6), each in a
  process group of its own; closing one kills its group, so nothing the
  box started outlives its stop.

### 7.3 What it names

- **Steps only:** each step's name, its arguments and whether it is
  once, long or link; and which of them are a forge's dormant group,
  by the forge's number alone. Its policy is a small machine of its own
  (steps, exits, timers and the pipe's lines in; spawns, signals,
  closes, the pipe's lines and the groups' records out), hosted on
  skein-io.
- **The steps in groups:** `Steps { always, asked }`, `always` the steps
  run in order, and `asked` a list of `Group { forge, before, steps }`,
  each dormant until asked (7.1). `before` is its place in the order:
  the index in `always` it runs just before (its length puts it last).
  Each step carries its `Stage`, whose word its `step` lines carry. The
  machine's step tests drive a group from dormant to `started`, and to
  `failed`, by lines and exits alone (13.1).
- **The binary hands it the steps.** `temper box` resolves its three
  directories, runs `temper init` when the configuration directory is
  empty (7.1), checks the document as section 8 says, asks `temper-fake`
  for its steps, `temper-forge-setup`'s with the demo's around them
  (8.4), and gives them to `temper-box`, an asked forge's group apart.
- **Checked:** a boundary test fails the gate when `temper-box` depends
  on a crate named `temper-*` but `temper-forge-setup`, or names an item
  of it but `steps`, `Steps`, `Group`, `Stage`, `Step` and its `Kind`.

### 7.4 Other arrangements of the same commands

Each needs nothing below the commands to change. None has the box's
pipe: `temper serve` runs without `--box`, an owned forge is set up by
running its commands, and the client names them (6.5).

- **systemd,** a unit per long step: `fetch` and `prepare` as
  `ExecStartPre=` of Forgejo's unit, `run` and `temper serve` as
  `ExecStart=`, the order as the units' dependencies; `temper serve`'s
  unit keeps systemd's default kill of its control group, which ends a
  crashed service's whole tree before its restart (skein's `shell.md`,
  13);
- **the process tier** (section 13.4), which runs the commands itself;
- **QA's environments** (`../draft/qa.md`, 4.1), whose "temper in a box" is
  an owned setup with a scripted LLM, made by its own host from the same
  commands, its LLM key added from the client once signed in
  (`../client/tools.md`, section 10);
- **the QA loop's cluster** (`../draft/qa-loop.md`, sections 4 and 11): the same
  `temper fake llm` and `temper fake forge` programs, with the loop's
  additions (deliveries, git and TLS with skein's test root; 8.4), beside
  `temper serve`, and a seed through `temper ctl`;
- **a container image** whose entry point is `temper box`, every process
  of its tree ending with the container.

## 8. The configuration

- **One document,** `temper serve`'s, `temper.json` in the
  configuration directory (configuration.md, sections 2 and 12). Every
  `temper forge` command and `temper box` read it whole, check it
  as configuration.md, section 8 says, `setup` among it, and act on
  their part: they read no file but their own part's.
- **`application.setup`,** a list, at most one entry per forge:

  ```
  "setup": [
    {"forge": 1,
     "bot": {"login": "temper", "email": "temper@example.invalid"},
     "mode": "managed"},
    {"forge": 2, "directory": "forge-2",
     "bot": {"login": "temper", "email": "temper@example.invalid"},
     "mode": {"owned": {
       "forgejo": "/opt/forgejo/forgejo",
       "runner": {"binary": "/opt/forgejo/forgejo-runner",
                  "labels": ["temper-host:host"], "capacity": 1,
                  "path": "/usr/local/bin:/usr/bin:/bin"},
       "listen": {"socket": "127.0.0.1:3000"},
       "person": {"login": "ada", "email": "ada@example.invalid",
                  "orgs": ["work"]}}}},
    {"forge": 3, "directory": "demo/forge",
     "bot": {"login": "temper", "email": "temper@example.invalid"},
     "mode": {"fake": {
       "listen": {"socket": "127.0.0.1:8369"},
       "git": "demo/bin/git",
       "owner": "demo",
       "repositories": [{"name": "widget", "landing": "main"}]}}}
  ]
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `forge` | u16 | the `number` of one entry of `forges`, of kind Forgejo |
  | `directory` | a data path (configuration.md, section 12) | when owned or fake, and only then: a private directory (section 3), unique among `setup` |
  | `bot` | `{"login", "email"}` | a Forgejo login; an email, the bot's when reconcile makes it (5.6) |
  | `mode` | `"managed"`, `{"owned": ...}` or `{"fake": ...}` | managed: temper may be given a site admin's token for the forge from the client (5.5); fake: the demo's (8.4) |
  | `start` | absent, or `"asked"` | when owned, and only then: absent, the box runs the forge's steps at every start; `"asked"`, they wait, dormant, until the client asks for them or the box's record says they ran (7.1) |
  | `owned.forgejo`, `owned.runner.binary` | cache paths (configuration.md, section 12), or `"fetched"` | checked against the pins of 6.1; `"fetched"` names temper's pinned build in the cache, which `temper forge fetch` writes (6.6) |
  | `owned.mirror` | absent, or `{"forgejo", "runner"}`, each an origin | each `https://HOST` or `https://HOST:PORT`, no path: where `temper forge fetch` asks for the pinned paths (6.6) |
  | `owned.runner` | the runner's members, or null for none | `capacity` at least 1; `labels` at least one, at most 8 |
  | `owned.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and owned and fake forges |
  | `owned.person` | `{"login", "email", "orgs"}` | a Forgejo login other than the bot's and `temper-admin`; `orgs` at least one, at most `setup.orgs`, each a Forgejo org name |
  | `fake.listen` | an address, jig's format | as `owned.listen` |
  | `fake.git` | a data path | the demo's git, a link to `temper fake git` (8.2, 8.4): the program the engine names in each of the forge's repositories' records, for the worker to spawn in place of `workers[].git` (channel.md, section 4) |
  | `fake.owner` | a Forgejo login | other than the bot's and `temper-admin`: the user who owns the fake's repositories |
  | `fake.repositories` | a list of `{"name", "landing"}` | at least 1, at most 8, each `name` a Forgejo repository name, unique ignoring ASCII case as Forgejo's are, and `landing` a branch name; their contents are `temper-fake`'s (8.4) |

  The code's entry keeps `orgs` at the entry and the fake entry has none,
  until the entry's form is brought to this table (temper-15).

- **A Forgejo with no `setup` entry** is connected's alone; GitHub takes
  no entry (section 17).
- **The forge's own members name no credential:** its records hold
  them (section 3). They stay connected's, written out, never derived,
  and checked against its setup; a mismatch refuses every command,
  naming the path: when owned or fake, its `api` is `listen`, its
  `transport` plaintext and its `git` `http://<listen>`, which a fake
  forge does not serve yet (8.4); an identity's `login`, when given, is
  the bot's.
- **A forge's `number` is its stable id** (configuration.md, section
  3), never reused: its records are kept under it, so a forge given
  another number is a new forge, not connected; and records made for
  another kind or address leave a forge not connected, named, with
  nothing erased (section 3).
- **`webhooks`,** `application.webhooks`, `{"listener": address}`, on
  loopback; each Forgejo forge's secret is its `WebhookSecret` record
  (configuration.md, section 3; skein's `forge.md`, section 7).
- **The clients' listener at a fixed address** when a GitHub forge is
  configured: GitHub's redirect and setup URLs name it, so its port is
  never 0, checked at start (17.1).
- **`temper serve` checks `setup`,** and reads one file of it, an owned
  or fake forge's handover (section 3). Under the box, with `--box`, it
  adopts the box's pipe, over which it asks for an owned forge's steps
  whose `start` is `"asked"` (7.1).
- **`application.demo`,** the demo's fake LLM and seed, or null for none
  (8.4).
- **An owned example** ships for a person, a test or QA's host to start
  from (8.1).
- **`temper init`** writes a deployment that starts (8.2), with the
  default document (8.3).

### 8.1 The owned example

`docs/examples/owned/` is a configuration directory (configuration.md,
section 12). Its document, `temper.json`, is a whole one, `engine`,
`workers` and `application`, for one owned forge with a runner,
webhooks, one git identity, one worker and two LLM providers, with no
LLM account: a person adds one from the client once the deployment has
started (credentials.md, section 3). Beside it are the files it names
that hold no secret: the agent's configuration, `agent.json`, and the
charters, `charters/`, short texts a person edits. The files that hold
a secret, the worker's secret and its digest, are data paths and never
shipped: `docs/guide/box.md` says how each is made in the data
directory. A person, a test or QA's host starts from a copy, edits the
agent's absolute paths to the copy's, and `temper box --config COPY`
runs it (section 7).

- **Its forge,** forge 1, is owned, in `forge-1`, with a runner whose
  label is `temper-host:host`, the bot `temper`, and the person `ada`,
  whose org is `acme`. Its members are its setup's, written out (above):
  `api` and `git` from `listen`, plaintext; no credential, since its
  records are made at the engine's first start, from the handover
  (6.5); the identity's `login` the bot's.
- **Its landing rule** covers `acme/widget`'s `main`, and asks for CI,
  Actions on the owned runner, the head up to date with its base, and an
  agent's review, gate 1, at a clean head (configuration.md, section 3).
  `acme/widget` is a repository the person makes in the org on
  Forgejo's web, with Actions, then adopts (6.5).
- **Its owner may do the forge's work in the org,** and no more: the
  rules' ceiling, the project's ceiling, its role's authority and its
  projections each hold one grant on the org, `acme`, for each kind a
  change and its goal's issue use (`../domain/forge.md`, section 6):
  read, push, open a pull request, merge, comment, create an issue, edit
  a pull request, update it from its base, close or reopen it, and edit,
  close or reopen an issue; 10, as the process tier's owner holds them.
- **Its LLM providers** (configuration.md, section 2), written out
  whole: 1, OpenAI, with keys only, its public Responses endpoint and
  gpt-6-luna with its window, output and prices. ChatGPT sign-in is
  added after verification against the real issuer (JP13). The charters
  allow the provider's models in order, the default the first whose LLM
  provider has a usable account (llm.md, section 5; jig's
  `domain/tasks.md`, section 3).
- **Its agent's configuration** (configuration.md, section 5),
  `owned/agent.json`, declares OpenAI's endpoint and model, and
  Claude Code's program, its path and its programs directory, a
  directory the person names; and takes the process tier's profile, the
  sizes its live tier runs a real model with: smith's `standard`, with
  `declared` `memory` 2,147,483,648, `tool_payload` 4,098, `read_window`
  256, `conversations` 1 and `calls_per_response` 1, and `policy`
  `max_turns` 8, `check_timeout` 60,000 ms, `group_stop` 10, `inbox` 8
  and `unacknowledged` 8. Both charters require eight turns; each permits
  modifying work with a 60,000 ms reserve, so the agent's check timeout
  must fit that reserve. The engine requires 4,096-byte messages;
  smith's two-byte rendered separator makes the minimum payload 4,098
  bytes, advertising 4,096 message bytes. Its largest call is then
  4,098 bytes, the larger of that payload and its 4,096-byte outcome
  (configuration.md, section 7, `small` against the agent). smith's
  standard sizes with nothing declared refuse the example's model
  declaration at start.
- **Its `engine.limits`** are jig's small profile with the overrides
  configuration.md, section 7 asks so that the engine's documents, its
  watch room and the forge's paths fit temper's client, and those its
  owner's grants need, and no other:
  - `authority.segments`, `tasks.hold_segments` and
    `tasks.authority_segments`, 8: a name's segments, the forge's
    deepest path;
  - `documents.rows` 2, `documents.lines` 2 and `documents.text_bytes`
    128: temper's client's rows and text;
  - `views.watchers` and `views.watchers_per_party`, 16: the watch room,
    as the root holds it whatever `engine.limits` says
    (configuration.md, section 7);
  - `authority.grants` and `tasks.authority_grants`, 12: its owner
    holds a grant for each of the forge's kinds its work uses, 10
    (above), and small's 4 hold none of that work; with `notes.scopes`
    15, since jig's core admits an authority's grants only with 3 notes'
    scopes more, and refuses every override of the core's otherwise
    (`jig-domain-core`'s `worst_case`);
  - `tasks.authority_bytes` 512 and `tasks.batch` 1, the process
    tier's: the bounds above widen a task's stored row past the small
    profile's store, whose bounds no override changes, so the start
    would be refused (jig's `protocol/store.md`, 5.4); these two bring
    the row back within.
- **Its worker's `limits`** are jig's small profile with the two of the
  hub's that jig's small, sized for any agent, sets past smith's agent
  (configuration.md, section 7, the worker's check against the agent),
  and no other: `hub.accounts` 4, the agent's accounts, where jig's small
  has 8; and `hub.event_bytes` 4,096, the agent's advertised message
  bytes, where jig's small has 65,536. Its `application.limits` is
  `small` with `checkout.message_bytes` 4,098 so the checkout accepts
  the agent's largest call.
- **Its `memory_bytes`** hold its worst cases wherever a person lays it
  and whatever its forge's records hold: each start passes with each
  string its service's worst case counts, and a person's copy changes,
  at its bound (configuration.md, section 7, one worst case per
  service).
  - The engine's counts the forge's API token, its `ApiToken` record,
    and its `git` base, a byte for each of theirs: the token at
    `small`'s `forge_component.token_bytes`, 512, past which the start
    is refused, and the base at an owned forge's longest, `http://`
    with a loopback address and a port, `http://127.255.255.254:65535`,
    28 bytes.
  - The worker's counts its agent's `program` and the configuration's
    path among its `arguments`, two bytes for each of theirs with the
    example's one slot: each at Linux's `PATH_MAX`, 4,096 bytes; and
    Claude Code's path and programs directory, which the host's half
    holds to check a program's login (smith's `protocol/programs.md`,
    section 4), each at the same bound.
  - Nothing else a copy changes moves either: its directory but for that
    path, its other addresses and secrets, its logins, emails, binaries
    and charters, its LLM providers, whose sizes jig's limits bound.

  Channel frames and queues use max(the checked credential value bound,
  the 32-byte source name, the generated Identity bound), independently
  on each side; configured keys include jig's 4,096-byte ceiling, and
  workers reserve it without provider declarations. The program paths
  are retained once, separately from the per-slot spawn command paths.
  `temper-service::startup::memory_requirements` derives both envelopes
  from startup's checked calculations without starting a service. The
  owned startup test checks the two literals against that query with
  these strings at their bounds in service settings, and checks each
  one-byte deficit at startup. Artificial paths are never opened. A
  separate shell test prepares the example with ordinary paths.
  Regenerate only the two literals with:

  ```sh
  SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --test startup -E 'test(owned_memory_figures)'
  ```

  At these pins the engine requires 79,538,334,051,202 bytes and the worker
  2,343,588,716,154 bytes. Ordinary runs fail on drift and never write the
  example. Program declarations survive startup; runnable program grants
  still wait for smith's program backend and jig's program sources.
- **Three loopback addresses,** Forgejo's (`listen`, `127.0.0.1:3000`),
  the clients' listener's (`127.0.0.1:7100`) and the webhooks'
  listener's (`127.0.0.1:7101`), are what a host changes to run several
  boxes on one machine, with each box's directories.

### 8.2 `temper init`

`temper init [DIRECTORIES]` writes a deployment that starts, from its
three directories (configuration.md, section 12) and what it finds on
the machine. `temper box` runs it first when the configuration
directory is empty (7.1).

- **Refused,** exit 1, before it writes anything, naming the file: when
  the configuration directory holds `temper.json`, or the data
  directory holds `worker.secret`, `worker.digest`, `demo/llm.key` or
  `demo/bin/git`. It never replaces a file; a person who starts again
  removes them.
- **Its directories:** the data and cache directories made and checked
  as every command makes and checks them (configuration.md, section
  12), and the configuration directory made the same way when absent;
  an existing configuration directory is taken as it is; and the data
  directory's `demo/` and `demo/bin/`, made private (0o700).
- **Its files,** each created new, in this order, the document last, so
  that a configuration directory holding one holds them all:

  | File | Directory | Mode | What it is |
  |---|---|---|---|
  | `worker.secret` | data | 0o600 | 32 bytes from the kernel's random source, as 64 lowercase hexadecimal digits and a newline |
  | `worker.digest` | data | 0o600 | the SHA-256 of the secret's digits without its newline, as 64 lowercase hexadecimal digits and a newline, as `docs/guide/box.md` makes it |
  | `demo/llm.key` | data | 0o600 | the demo model's key (8.4): 32 bytes from the kernel's random source, as 64 lowercase hexadecimal digits and a newline |
  | `demo/bin/git` | data | a link | the demo's git, `temper fake git` (8.4): a symbolic link to the binary |
  | `charters/chat.txt`, `charters/coding.txt` | configuration | 0o644 | the charters' instructions, the owned example's |
  | `agent.json` | configuration | 0o644 | smith's agent configuration (configuration.md, section 5): the owned example's endpoints, models and profile (8.1), and the demo's endpoint and model (8.3); Claude Code's program at the `claude` found on `PATH`, and its programs directory, `programs` in the data directory; with no `claude` on `PATH`, no program, and Anthropic's models by key alone |
  | `temper.json` | configuration | 0o644 | the default document (8.3) |

- **What it finds,** each written absolute:
  - *the binary,* the running `temper`'s own path: the worker's agent's
    `program`, with `arguments` `["agent", <configuration>/agent.json]`
    and `configuration` that same path (configuration.md, section 5),
    and the demo's link's target;
  - *git,* the `git` found on `PATH`: the worker's `git`; with none,
    `null`, written without refusing, and init says so (git.md, section
    3). The demo's link, `<data>/demo/bin/git`, is the fake entry's
    `git` (section 8), so that the demo's git is `temper fake git`
    (8.4) whatever git the machine has, or none;
  - *the person,* forge 3's (8.3): its `login` the name of the effective
    uid's entry in `/etc/passwd` (`geteuid`, then that file), the login
    the person signs in to their Forgejo with; its `email`
    `LOGIN@localhost`; its one org `work`. With no entry, or a name that
    is not a Forgejo login, is one of Forgejo's reserved names (such as
    `admin`), or is the bot's, `temper-admin` or `work`, the login is
    `person`, and init says so: forge 3 waits dormant, so the person's
    login is never a reason the first run does not start;
  - *the bundle,* `<the binary's directory>/../share/temper/bundle` when
    it holds `index.html`: `clients.bundle`; else `null`, and init says
    the browser is not served;
  - *the clients' listener's port:* 8367, when a socket binds
    `127.0.0.1:8367`, else the port the kernel assigns to a bind of port
    0; the port written fixed, as a GitHub forge requires
    (configuration.md, section 3). A later start on a port taken is
    refused, naming it and the document, as any start is;
  - *the forges' ports:* 8368, forge 3's, the owned Forgejo's (8.3);
    8369, the fake forge's, and 8370, the fake LLM's (8.4), each by the
    same rule. The four sockets, the clients' and these three, are held
    until all four ports are chosen, so that no two are the same, then
    closed.
- **What it says,** on stderr: `init: wrote <path>` for each file;
  `init: no bundle at <path>: the browser is not served` when there is
  none; `init: no git on PATH: a real forge's repositories need git
  2.41 or later, set as workers[].git` when there is none; `init:
  LOGIN cannot be a Forgejo login: forge 3's person is person` when
  the login falls back (above); and last,
  `init: start it with temper box`, with the flags init was given. Its
  stdout is empty; exit 0.
- **Nothing else:** no account, no forge credential, no repository and
  no store. The engine's first start makes its store; accounts, forges
  and repositories are the client's (credentials.md, section 3;
  `../client/forge.md`), and the demo's account and repository the
  seed's, through the client's protocol (8.4).
- **Its code:** `temper-init`, step code (section 11): given the facts
  above, git's path or none and the person's login among them, the three
  directories, the secret's 32 bytes and the demo key's 32 bytes, it
  answers the files, each with its directory, its name, its bytes and
  whether it is private, and the person's login, fallen back to `person`
  or not; it reads, writes and binds nothing. The binary's `init` finds
  the facts, checks the refusals and writes the files, on skein-io.
- **Checked:** temper-init's step tests check that its `engine.llm` is the
  owned example's with the demo's provider (8.3), that the digest is the
  secret's, and the files' names, directories and order. The binary's
  crate tests, on scratch directories, check that the document init writes
  passes configuration.md, section 8, as `temper serve` checks it, with
  the agent's configuration it writes, at the fixed ports and at kernel's
  ports, the four always distinct, with and without `claude`, a bundle and
  git; each refusal writing nothing, the person's fallback login among
  them; the files' modes; and the link's target. The process tier starts a
  box on empty directories (13.4).

### 8.3 The default document

A working minimum, for the mechanics and a person's first run; good
defaults for a person's real work are open (section 16).

- **Its forges,** each by its stable number (configuration.md, section
  3):
  - 1, the demo forge, Forgejo, its members its setup's (section 8):
    `api` `127.0.0.1:<the fake forge's port>`, plaintext, and one git
    identity whose `login` is the bot's, `temper`;
  - 2, GitHub, at GitHub's addresses, with one git identity whose
    members are GitHub's defaults, not connected until the person
    connects it from the client (section 17);
  - 3, the person's own Forgejo, its members its setup's (section 8):
    `api` `127.0.0.1:<its port>`, 8368 (8.2), plaintext, `git`
    `http://127.0.0.1:<its port>`, and one git identity whose `login`
    is the bot's, `temper`; dormant, not connected until the person sets
    it up from the client (`../draft/first-run.md`, 3.8; 6.5).

  GitHub and the owned Forgejo are in the document from the first run,
  so that connecting either from the client needs no restart.
- **Two `setup` entries:**
  - forge 1's, fake, in `demo/forge`, its `git` the demo's link (8.2),
    its bot `temper`, its `owner` `demo`, its one repository `widget`,
    landing on `main` (8.4);
  - forge 3's, owned, in `forge-3`, `"start": "asked"` (7.1): its
    `forgejo` and `runner.binary` `"fetched"` (6.6); its runner the
    owned example's, `temper-host:host`, capacity 1, and its `path`;
    its `listen` `127.0.0.1:<its port>`; its bot `temper`; and its
    person the one init finds, `LOGIN@localhost`, with the org `work`
    (8.2).

  No webhooks' listener: reconcile registers no hook, so the fake has
  none to deliver to, forge 3 polls (forge.md, section 6), and GitHub's
  hints come from its App's delivery log.
- **Its LLM providers:** the owned example's `engine.llm`, whole (8.1),
  with no account; and 3, Demo, the demo model's (8.4): one endpoint,
  `demo`, at `127.0.0.1:<the fake LLM's port>`, plaintext, OpenAI's
  dialect on its public route; one model, `demo`, priced as the owned
  example's model is, 1 per 1,000,000 for input, cached and output (jig
  and smith refuse a price of 0); its window 131,072 and its output
  8,192, so that smith's room at the owned threshold, 4/5, still offers
  the owned message bound, 4,098 bytes, and a request at the threshold
  stays within the fake's request bound; its source a key. The agent's
  configuration declares it too (8.2).
- **One project,** number 1, adopting nothing until the person adopts
  a repository from the client. Its landing rule covers every
  repository of every forge, the pattern `{"segments": ["forge"],
  "last": "open", "value": ""}`, `enforce`d, with `ci`
  and `up_to_date`, and no gate or approval: a change lands on green CI
  with no person in between. Its owner's grants, the rules' ceiling and
  its projections are the owned example's, on that pattern in place of
  the org's.
- **Its charters and templates,** the owned example's, but that chat's
  models end with `demo`: chat, 1, and coding, 2, the one the forge's procedures start agents' work under
  (`application.charter`); the templates `chat` and `goal`.
- **One worker,** in the process, its `root` `workspaces` in the data
  directory, its `git` the machine's, or null (8.2); the deployment's directory
  `deployment` in the data directory; the clients' listener
  `127.0.0.1:<port>` (8.2).
- **Its limits:** the owned example's (8.1). Its `memory_bytes` are
  derived as the owned example's are, with its own strings at their
  bounds, the fake entry's `git`, which the engine names to the worker,
  among them; two literals of `temper-init`'s checked by its tests the
  same way.
- **`application.demo`,** the demo's (8.4).
- **Its box's steps:** the fake LLM, forge 1's preparation and its fake,
  `temper serve` and the seed (7.1); and forge 3's group, dormant until
  the client asks for it: its fetch, its preparation, Forgejo's web and
  the runner's daemon.

### 8.4 The demo

A person's first run is a demo (`../draft/first-run.md`, 3.5, 3.6 and
5, step 3): the box runs a fake LLM and a fake Forgejo from temper's
own binary, the engine sets itself up on the fake as on an owned one,
and a seed adds the demo model's key and adopts the demo's repository
through the client's protocol. Its bar: a chat that works on the demo
model, and the demo forge connected and its repository adopted. Its git
is `temper fake git`, on the fake forge's git route, so a worker's
clone and push reach the fake; the demo model answers in text and
finishes, so the demo's own work pushes nothing yet.

- **`application.demo`,** or null for none:

  ```
  "demo": {
    "llm": {"listen": {"socket": "127.0.0.1:8370"}, "provider": 3,
            "key": "demo/llm.key",
            "asked": [{"cue": "Add CSV export to widget",
                       "turns": ["..."]}],
            "otherwise": ["..."]},
    "forge": 1, "project": 1, "client": "demo/client"}
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `llm.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and owned and fake forges |
  | `llm.provider` | u32 | the number of one of `engine.llm`'s providers (configuration.md, section 2), whose endpoints are each at `llm.listen`, plaintext, OpenAI's dialect on its public route, and whose source is a key alone |
  | `llm.key` | a data path (configuration.md, section 12) | a secret's file, 0o600 (section 9): the key is its bytes less a final newline |
  | `llm.asked` | a list of `{"cue", "turns"}` | at most 8; each `cue` non-empty text of at most 256 bytes, unique, the words of a request the client suggests (`../client/README.md`, 4.8); `turns` 1 to 8 turns (below) |
  | `llm.otherwise` | a list of turns | 1 to 8 |
  | `forge` | u16 | the `forge` of one fake `setup` entry (section 8) |
  | `project` | u32 | the number of one of `engine.projects` (configuration.md, section 2) |
  | `client` | a data path | a private directory (section 3), unique among the document's directories |

  The scripts, `asked` and `otherwise` together, fit skein's fake LLM's
  `script_bytes` (skein's `fake-llm.md`), which `temper-fake` sets.
- **A turn** is skein's `Turn` (skein's `fake-llm.md`, section 2), in
  either of two spellings:
  - *a text:* one text line, finishing `stop`, its tokens its bytes
    divided by 4, rounded up;
  - *an object,* `{"lines", "finish", "tokens"}`, the QA loop's and the
    demo's: `lines`
    1 to 8, each `{"text"}`, non-empty text, or `{"call", "arguments"}`,
    a tool's name of 1 to 64 ASCII letters, digits, `_` and `-` and its
    arguments a JSON object, which `temper-fake` writes compact as the
    call's bytes; `finish` one of `stop`, `tool_calls`, `length` and
    `content_filter`; `tokens` a u64, at least 1.

  Each is checked as skein checks a script's turns (its section 2.3).
  No turn holds a hold, a failing line or opaque bytes.
- **Its words,** as `temper init` writes them (8.2): one `asked` script
  for each request the client suggests in the demo project
  (`../client/mockups/first-run.html`): "Add CSV export to widget", "Add
  a dark theme to widget" and "Fix widget's empty search state", each
  answering in text what it would change and why; and an `otherwise`
  that says it is the demo model, which answers only the suggested
  requests, and how to connect a real one from the client's accounts
  page. Each is one turn, an object: its text line, then
  `{"call": "finish", "arguments": {"form": "report", "text": "The demo
  changes nothing.", "fields": {}}}`, finishing `tool_calls`, its
  tokens its text's bytes divided by 4, rounded up. A chat's run ends
  only at `finish`: a text-only stop is nudged once, and a second fails
  the run (smith's `domain/run.md`, 7.2), and the chat's contract is a
  report of at most 32 bytes (`engine.templates`' `chat`), so the answer
  is the text and the report is short. `finish`'s arguments are the
  contract's, as smith's schema for the tool has them (its
  `smith-protocol-llm`'s `contract.rs`): a `form`, the report's `text`
  and its `fields`, none for a chat. Their content is the demo's
  follow-up (`../draft/first-run.md`, section 8).
- **`temper fake llm DIRS`,** a long step: skein's fake LLM peer, run
  `without_records` as a peer serving a person is (skein's
  `fake-llm.md`), its domain `Domain::played` with `asked` as its
  cued scripts and `otherwise` as its fallback, no asked script and no
  menu. A chat's opening words are its run's brief, which smith puts in
  the system text, and the one user message is smith's own opening, so
  a request's cue is in the system, where skein's cued scripts look and
  its asked ones do not (skein's `fake-llm.md`, 2.4). A cued script's
  turns count every assistant message of the conversation, so a chat's
  first answer is its script's first turn, and its one turn finishes
  the run: the chat is then ended, with no composer, so no second
  message reaches the demo (a message sent while the run is active
  would join the same conversation, past the script's turns, and be
  answered "done"). It reads the
  key file before it binds, and refuses to start without it, naming
  the file; it accepts that one credential, and refuses every other
  request as the provider would. It serves `llm.provider`'s models, at
  their windows and outputs; no failure and no latency is configured,
  and its limits are `temper-fake`'s constants.
- **`temper forge prepare DIRS N` on a fake entry,** a once step: its
  directory made private; `admin_token` written when absent, 20 bytes
  from the kernel's random source as 40 lowercase hexadecimal digits,
  as Forgejo's tokens are. The fake reads it before it binds, so the
  handover is the preparation's, not the fake's own at its start, as
  the draft had it: no step races `temper serve` for the file.
- **`temper fake forge DIRS N`,** a long step: skein's fake forge peer,
  `without_records`, at `fake.listen`. It reads `admin_token` before it
  binds, refusing to start without it, naming the file, and leaves the
  file for `temper serve`. Its seed:
  - *temper's admin,* `temper-admin`, a site admin of the setup's
    routes, its one token the handover's, with every scope (skein's
    `fake-forge.md`, section 4);
  - *the owner,* `fake.owner`, a user of the domain, numbered apart from
    the setup's accounts (skein's `fake-forge.md`, section 5);
  - *its repositories,* `fake.repositories`, each the owner's, in the
    protocol's `config.repositories` and the domain alike, numbered
    from 1 in the list's order at every start, so the box's
    `demo/widget` and the QA loop's `fake/widget` are each 1; on its
    `landing` branch, one commit of `temper-fake`'s constants, its
    files, author and time alike, so its head is the same commit at
    every start: a few files and `.forgejo/workflows/check.yml`, which
    describes the check the fake's CI stands for and is not run;
  - *its CI,* `Checks` with one context, `check`, with no latency,
    silence or rerun: on a repository with a cue, the domain's `Cue { path, green
    }` (skein's `fake-forge.md`, 2.3), a head whose file at `path` holds
    `green` passing and any other failing; on one without, every head
    passing. The box gives none; the lab's form gives each repository's
    (below).

  The bot is reconcile's, and its access to a repository the adoption's
  collaborator grant, which reaches the fake's domain (skein's
  `fake-forge.md`, section 4): the seed grants nothing. It serves its
  git route among its routes (skein's `fake-forge.md`, section 4), and
  no git smart HTTP; it keeps no state across its starts.
- **Its deliveries follow the setup's hooks** (skein's `fake-forge.md`,
  section 4): each event goes to each hook a repository's setup holds
  that takes it, signed by that hook's secret. Reconcile makes a
  repository's hook at adoption when the deployment has a webhooks'
  listener (section 10), at `http://<listener>/hooks/<number>`, so the
  fake reaches serve's listener as a Forgejo does. The box has no
  webhooks' listener (8.3), so it registers none and polls; the QA
  loop's serve has one, on loopback.
- **`temper fake git`,** no step: the demo forge's git, which the
  worker spawns for the fake's repositories by the program their
  records name, the fake entry's `git`, the demo's link (8.2; channel.md,
  section 4), in place of `workers[].git`, as it spawns real git
  (git.md, section 5), and which it checks at its first use as it checks
  real git, by `git version` (git.md, section 3). A real forge's
  repositories, on the same worker, take the machine's git. It is the
  fake git program (`../domain/testing.md`,
  4.3): the fake checkout's `.git` in the worker's working trees; its
  remote the fake forge's git route, at the host of the clone URL the
  engine gives; its credential the bot's, from git's
  `http.extraHeader`; its store the fake's domain, so a push is the
  head the engine reads.
- **Its programs' flags,** for the box and the QA loop's runner alike
  (`../draft/qa-loop.md`, sections 4 and 11), after the operands:
  - `--ready PREFIX`: once its listener is bound, the program writes
    PREFIX, then the bound address as `127.0.0.1:<port>`, then a
    newline on stdout, flushed; nothing else is on its stdout;
  - `--seed N`: its domain's seed, a u64 in decimal (skein's
    `fake-llm.md`'s `Domain::played`; skein's `fake-forge.md`'s
    `Service::new`), 0 when absent, as the box runs it; in a document's
    form, its credentials' too (below);
  - `--observations FILE`, the fake forge's alone: FILE created new at
    0o600, its first line a checkpoint, `{"observations_lost":0}`,
    written before the ready line, and each observation its domain
    makes (skein's `fake-forge.md`, 2.4) drained after each pass, as a
    referee drains it, and written to FILE in skein's encoding, one
    JSON line each in drain order. A pass that observed or lost any
    ends its lines with a checkpoint, `{"observations_lost":N}`, the
    count lost so far, and the program writes one more at its stop, so
    FILE is whole up to its last checkpoint from its ready line on, a
    copy taken while it runs as much as the file, and a stopped
    program's last line is one. Nothing
    of a pass is sent, no reply and no delivery, before its lines'
    write completes, so a reader that has seen a reply, or what a reply
    caused, finds in FILE what the pass observed. Without the flag, the
    program drains and drops them, and sends as it serves.

  A write failure stops the program with exit 1. After the first failed
  write it writes nothing more to FILE, including at stop: it drains and
  drops observations and refuses every later pass's sends. A drain past
  its byte bound has the same rule. FILE retains its earlier writes and
  whatever bytes the failed write left, with no later checkpoint to
  certify lines it lost.

  Both peers run `without_records`, with the flags or without them.
- **From a document of its own,** `temper fake llm --document FILE` and
  `temper fake forge --document FILE`, the QA loop's form, which reads
  no deployment: FILE is a fake's configuration, a JSON object, as
  `temper-fake` derives it from the deployment's document in the box's
  form, with what a run needs beside it. The LLM's:

  | Member | Type | Bounds |
  |---|---|---|
  | `listen` | an address, jig's format | a socket address on loopback, port 0 for the kernel's |
  | `key` | an absolute path | the key's file, which the fake writes (below) |
  | `tls` | `{"root"}`, or null for plaintext | `root` an absolute path: the test root's file, which the fake writes (below) |
  | `models` | a list of `{"name", "window", "output"}` | 1 to 8, each as a provider's model's members are (configuration.md, section 2) |
  | `cued` | a list of `{"cue", "turns"}` | at most 8, as `llm.asked` is; each cue looked for in the system text (skein's `fake-llm.md`, 2.4) |
  | `asked`, `otherwise` | as `llm.asked` and `llm.otherwise` | the same |

  The forge's:

  | Member | Type | Bounds |
  |---|---|---|
  | `listen` | an address, jig's format | a socket address on loopback, port 0 for the kernel's |
  | `admin_token` | an absolute path | the handover's file, which the fake writes (below) |
  | `owner` | a Forgejo login | other than the bot's and `temper-admin` |
  | `repositories` | a list of `{"name", "landing", "ci"}` | as `fake.repositories` are; `ci` `{"path", "green"}`, or null for every head passing: `path` a file's path in the repository, relative, none of the seed's, and `green` non-empty text, both within the domain's content limits |

  The box's derivation gives `cued` none, `tls` and each `ci` null.
- **Its outputs,** in a document's form, where the box's fakes read
  the files `temper init` and the preparation drew: the fake writes
  each file the document names before it binds, so that each exists
  before its ready line, created new at 0o600; a file already there
  refuses the start, naming it. Each file's directory must exist, but
  the admin token's, the forge's directory (section 3), which the fake
  makes private when absent and refuses, naming it, when it is not
  private, as `temper forge prepare` does. The key and the admin token
  are each 20 bytes as 40 lowercase hexadecimal digits, from
  skein-lib's `Rng` seeded by `--seed` and a constant of each
  credential's, a stream apart from the domain's draws, so the runner
  draws none (`../draft/qa-loop.md`, section 4). A seeded credential is
  no secret from whoever holds the run's record: it opens only that
  run's loopback processes, which end with it, which is why the box
  draws its own from the kernel instead. `tls.root` gets
  `skein_fake_peers::transport::ROOT`'s bytes, and the fake serves the
  fixed chain that root signs, whose leaf names `skein.test` and
  `127.0.0.1` (skein's `fake-llm.md`, section 3); an endpoint reaching
  it names `skein.test` as its server and trusts that file alone
  (configuration.md, section 12), as the QA loop's serve and agent do.
  The fake removes none of them at its stop: serve's setup host
  removes the admin token when it takes it (section 3), and the run's
  directories go with the run.
- **A runner** starts the fakes first, learns their addresses from
  their ready lines, then writes temper's document naming them and the
  files the fakes wrote, and starts `temper serve`.
- **`temper serve`** takes the fake's handover and reconciles it as an
  owned forge's (5.1, 6.5): `TakeOver`, then `Reconcile`, which makes the
  bot and its tokens on the fake. A fake not yet bound when the start's
  reconcile reaches it is reconcile's wait for the forge (5.3).
- **`temper fake seed DIRS`,** a link step: `temper ctl` calls
  (`../client/tools.md`, section 10), each a child of the binary's own
  run, its DIRECTORY the demo's `client`. A call is the person's own
  tool, at its own exit statuses and lines, so the seed holds no client
  and reads only what each call prints. Its acts, in order, each reading
  first and done when it finds itself done:
  1. sign in with the link on its stdin, serve's first line;
  2. add the demo model's key, `llm.key`, as the deployment's account of
     `llm.provider`, its idempotency key `demo-llm`;
  3. wait until that account is usable, and until every forge is
     settled;
  4. adopt `fake.repositories`' first into `project`, from `forge`, the
     owner's, on its `landing`, its idempotency key `demo-adopt`;
  5. sign out;
  6. present the link's secret again: it is refused as spent, and
     serve writes a fresh link to its stdout, the box's, which is the
     person's (7.2).

  It exits 0. A seed that fails, at any act, makes the box give up,
  naming it (7.2).
- **A restart** of the box starts the fakes afresh: the preparation
  draws a new handover, the engine's start takes it and reconciles,
  which makes the bot, its tokens and each adopted repository's access
  again (5.3, step 6), and the seed finds its acts done but the sign-in
  and the link. The fake seeds its repositories the same at every
  start, so a worker's clone of one still matches it; a branch an agent
  pushed is lost with the fake's state, and the engine's records of it
  outlive it, until the fake keeps its state across starts (below).
- **`temper fake forge check DIRS N OBSERVATIONS STORE`,** no step: the
  QA loop's check of the fake forge's promises (`../qa.md`, 8.3),
  skein's judge (skein's `fake-forge.md`, 2.5) over forge N's
  observations, with temper's decisions from temper's own record, a
  copy of its store, never from the merges judged (decision 259). It
  reads the document for forge N, a fake `setup` entry, and for the
  store's limits; it reaches no peer and writes nothing but the copy,
  which opening may repair.
  - *OBSERVATIONS,* the file `--observations` wrote, or a copy of it:
    its lines through its last checkpoint, the flush boundary, whose
    count is the judge's loss. What follows is a flush still being
    written, and is not read. A file without a checkpoint, or a line
    before it that skein's decoder refuses at the fake's limits, is
    undecided: `observations missing`.
  - *STORE,* a copy of the deployment's store, `store/` in
    `engine.deployment.directory`, never the store itself, taken once
    serve has exited, stopped or killed: `/bin/cp -R -- <store>/.
    <empty directory>/`. A copy taken while serve runs is not one:
    skein's KV store has one process (skein's `kv.md`, section 1), and a
    copy made beside its writer can open to part of a commit with
    nothing refused. The lab takes it after the run's processes stop
    (skein's `lab.md`, its collection after the stop), so a run whose
    lab has no such step has no copy. It is opened as serve's store
    opens it, at the limits `temper-service` derives from the document,
    and its format checked as jig checks it, temper's interval read in
    key order; one that is absent, does not open, is of another
    `STORE_VERSION`, or has a row that does not decode is undecided:
    `store unreadable`.
  - *The profile,* temper's as a client of forge N: `client` forge N's
    `Writer`, the bot's number on the fake, and with none, undecided:
    `client unknown`; `marker` temper's key's delimiters,
    `<!-- temper:key ` and ` -->` with its line feed (forge.md,
    section 4); `owned_branches` the `prefix` of each of forge N's
    `Repository` rows; `allowed` the writes temper makes on items it
    did not open: `Comment`, `Merge` and `Close`, and `Review` and
    `Request` when one of those rows' `kinds` grants `review`, since a
    task reviews an outside contribution and sets its reviewers
    (`../domain/forge.md`, section 13), while an update of one stays
    foreign, since temper updates only its own change's branch
    (section 10 there); `bounds` the decisions' count, and
    `temper-fake`'s constant for the objects, past which the judge is
    undecided: `judge full`. The constant is sized for the runs the check
    judges, not for the fake's limits: the judge reserves each repository
    slot's tables at it, so its memory grows with its square, and a run's
    history, each branch and key it saw, is bounded by no limit.
  - *The decisions,* each merge temper decided on forge N, as the
    judge's `Decision`: its repository `owner/name`, its pull and its
    head. They are read from two families (store.md, section 2):
    - each `Entry` whose write is a merge, settled or not, sent or not:
      its `Write::Merge`'s number and head, and its repository;
    - each `Landed` row, a merge temper made, joined by its task to the
      task's `Change` row: that row's repository, `pull` and
      `last_head`, since a change's head stays put from its decision to
      its landing.

    A change in `Landing`, or held over it, adds none of its own: its
    `Entry` stands for it until the merge settles, and its `Landed`
    row after, if temper made the merge. A head is the fake's commit,
    its first 8 bytes big-endian and the rest zero; any other is
    undecided: `store unreadable`. So is a `Landed` row whose task has
    no `Change` row, or a `Change` row with no `pull` or `last_head`.
    Rows of other forges' repositories are skipped. The decisions may
    be incomplete when the `Landed` rows number the forge domain's
    `landings` limit (store.md, section 7), or when a decision's
    `Repository` row is gone. An `Order` breach is then undecided,
    `decisions incomplete`, not a fail, since a decision missing can
    only make a merge look undecided; the other breaches stand.
  - *Its order:* the decisions, then `decided`, then the observations,
    then the verdict, so no merge is judged before every decision is
    in. The collection copies OBSERVATIONS no later than STORE: a
    decision is saved before its merge is sent (`../domain/forge.md`,
    section 6), so a store copied after the observations holds the
    decision behind every merge they hold.
  - *Its verdict,* by the check's contract (skein's `lab.md`, 7.2;
    decision 259): a pass is status 0 and no line; breaches are status
    1, a `{"fail": LINE, "summary": TEXT}` line each, its `line()` the
    signature and its summary the promise's words, then an
    `{"undecided": REASON}` line when the evidence is also incomplete;
    undecided alone is status 2 and its line. The judge's reasons read
    `observations lost`, `judge full` and `observations unforwarded`,
    and the check's own `observations missing`, `store unreadable`,
    `client unknown` and `decisions incomplete`; with the decisions
    given first and the stream cut at a checkpoint, no merge precedes
    `decided` and no stream is truncated.
  - *Its stderr* is what it read, kept as its evidence, in JSON lines:
    `{"boundary":{"lines":N,"lost":L}}`, the checkpoint's line count
    and loss; `{"profile":PROFILE,"decisions_complete":BOOL}`, the
    profile's members by their names in skein's `Profile`; and
    `{"decision":{"repository":OWNER/NAME,"number":N,"head":H}}` for
    each decision, its head the fake's commit number. A reader's refusal
    follows as a text line.
  - *Its own errors:* a refused argument, a document that does not read
    or check, or a forge N without a fake entry is the check's error,
    not the run's: status 1, its refusal on stderr and no line, which
    the lab reads as the check's error, never a pass (skein's `lab.md`,
    7.2). The check creates no directory.
- **Its code:** `temper-fake`, step code (section 11): the member's
  types and checks; the fake LLM's and the fake forge's configurations
  and seeds, from the deployment's document or from one of their own;
  `steps`, the box's steps for the document; and the seed's plan, its
  acts as `temper ctl` calls with what each reads to find itself done.
  It names no host. Its dependencies:
  `temper-forge-setup`, `skein-fake-forge-domain`,
  `skein-fake-forge-protocol`, `skein-fake-llm-domain`,
  `skein-fake-llm-protocol`, `skein-llm`, `skein-http`, `skein-forge`,
  `skein-fake-peers`, `jig-protocol-config`, `skein-lib` and
  `skein-json`; and for the check, `temper-protocol-store` and
  `temper-engine-domain-forge`, since the check's derivation is
  `temper-fake`'s too: from the store's rows to the judge's profile
  and decisions, and from its verdict to the check's lines. Its table
  fixtures also name `skein-forge-client`,
  `temper-engine-domain-forge-change` and
  `temper-engine-domain-forge-issues`. The binary's
  `crates/temper/src/fake.rs` hosts the peers and the seed on
  skein-shell, the seed's calls as children of `temper ctl`, reads the
  check's files and opens the store's copy at the limits
  `temper-service`'s `startup::store_limits` derives from the document,
  and `temper fake git` is `temper-fake-git`'s core on the real file
  system (`../domain/testing.md`, 4.3), which `main` runs when its
  `argv[0]`'s last component is `git`.
  `Mode::Fake` and the link step's kind are `temper-forge-setup`'s.
- **Not yet:** git's smart HTTP on the fake, its pages, and its state
  across starts; and a webhooks' listener in the box, so the demo's
  forge delivers as the QA loop's does; each is the demo's follow-up
  (`../draft/qa-loop.md`, sections 4 and 11).

## 9. Credentials

| Credential | Mode | Made by | Kept | Used by |
|---|---|---|---|---|
| a site admin's token | Forgejo, managed; owned; fake | the person, pasted; the preparation, by Forgejo's command line, or drawn for a fake, or the fake from its seed in a document's form (8.4) | the `AdminToken` record; an owned or fake forge's handover, until the engine takes it, which a fake forge reads at its start | the setup host alone, while a plan runs: reconcile, `TakeOver`, access at adoption (`Adopt`), and the picker's read (`Reachable`, skein's `forge-setup.md`, 4.8) |
| the bot's API and git tokens | Forgejo, every mode | the person, pasted; reconcile | the `ApiToken` and `GitToken` records | the forge component; the identities' table |
| the bot's password | Forgejo, managed, owned, fake | reconcile, random, when it makes the bot | nowhere | nothing |
| temper's admin's password | owned | Forgejo, random | nowhere | nothing |
| the person's first password | owned | Forgejo, random, at the first preparation | `person_password` | the person |
| the runner's secret | owned | the preparation | `runner/secret` | Forgejo's command line; the runner's connection, through the token's file it names |
| Forgejo's own secrets | owned | the preparation | `state`, and `app.ini` | Forgejo |
| the webhooks' secret | Forgejo, every mode | the person, pasted; reconcile, drawn | the `WebhookSecret` record | the webhooks' listener; Forgejo's hooks |
| a GitHub App's key | GitHub | GitHub, at the manifest's conversion; the person, pasted as PEM | the `AppKey` record, DER; a PEM never | the forge component, for its tokens; the setup host, while a plan runs |
| an installation's token; a GitHub git token | GitHub | the forge component, from the App's key | memory alone | the forge component; git |
| the demo model's key | the demo | `temper init`, random (8.2), or the fake LLM from its seed in a document's form (8.4) | `demo/llm.key`, or the document's `key` | the fake LLM, its one credential; the seed, which adds it as the deployment's account (8.4) |

- **Every credential the engine holds is a record** in jig's secrets'
  store (credentials.md, the records), written by the setup host's
  plans; a pasted value is kept by jig until its run ends
  (credentials.md, pasted values).
- **Least power, kept.** The forge component holds the bot's tokens and
  the webhooks' secret, or the App's key, as in connected, and never an
  admin's token; the bot is never a site admin; the setup host holds the
  admin's token only while a plan that asked for it runs, and drops it
  at its end.
- **No secret in an argument, a URL or a log,** for Forgejo's command
  line as for git (credentials.md, section 4): 6.3.
- **No new credential to set up from the client:** `temper forge
  fetch` sends none and keeps none, its downloads trusted by their
  pinned digests alone (6.6); the box's pipe carries no secret, a
  failure's reason among its lines (7.1); and the handover is taken as
  at a start (6.5).
- **Never across a domain boundary:** the setup host is in the engine's
  protocol layer, below the core, which sees a pasted value's number and
  digest alone; the engine's rules (credentials.md, section 1) hold for
  the records and for the handover.

## 10. Webhooks on a box

The listener is skein's (`forge-connection.md`, section 6), and so are
the signature, the events and the hints they become (`forge.md`, section
7). This section says what makes them reach the engine on a box, and
that they cover what its work waits on.

- **Made at adoption, checked at each start** (section 5; skein's
  `forge-setup.md`, 4.5, and 4.3, step 6): one hook per adopted
  repository, to `http://<listener>/hooks/<number>`, with the events of
  skein's `forge.md`, section 7, and the forge's secret. An owned
  Forgejo's `app.ini` allows the listener's address (6.2). On a box,
  nothing is left to the person.
- **What a box's work waits on, and what ends each wait,** with the keep
  reading the forge within `hinted` of the hint, which gathers one
  change's hints into one read (`../domain/forge.md`, 5.2):

  | The domain waits on | Done on the forge by | Its delivery | Its hint |
  |---|---|---|---|
  | a landing branch's tip | a merge, a person's push | `push` | the branch moved |
  | a pull request's head, base or state | a push to its head or base; closing, reopening, retargeting or editing it | `push`; `pull_request` | the branch moved; the item |
  | CI on a head | each Actions job ending | the job's end | the commit |
  | a participating object's comments and reviews | a comment, a review, an approval or a rejection | the item's comment or review events | the item |
  | an issue of a live goal | editing, closing, labelling or assigning it | `issues` | the item |
  | drift of a branch temper holds | a push, a deletion, a creation | `push`; `delete`; `create` | the branch moved; the branch |

- **Echoes go up too.** Forgejo delivers temper's own writes. The client
  reads on every hint it holds a resource for, so each costs one read;
  temper, the owner, drops its own echoes by key and by pusher (skein's
  `forge-client.md`, section 4; `../domain/forge.md`, 5.2).
- **What Forgejo settles after it delivers,** a pull request's
  mergeability, which it answers not mergeable while it checks (skein's
  `forge.md`, section 9), is read again at `hinted`, doubling, within
  `settle` of the move, as is a hinted job's CI that reads pending.
  These are the owner's fresh reads: the client runs no window (skein's
  `forge-client.md`, sections 4 and 5). A pull request that reads not
  mergeable within `settle` of the move is never taken as conflicted
  (`../domain/forge.md`, 5.2).
- **What sends no delivery,** and so is found by polling alone: a
  commit status that a system other than Actions posts, which a
  box has none of.
- **Polling is unchanged** (skein's `forge-client.md`, section 4): the
  keep's alarms, its paces and the slow pass are what they are with no
  webhook. A delivery lost while `temper serve` is down, refused for its
  size, or never sent is found by them; Forgejo does not send a failed
  delivery again (skein's `forge.md`, section 9).
- **GitHub sends no delivery to temper:** its App's changes come from
  the App's delivery log, read by the forge component, with no address
  GitHub must reach and no webhook secret (forge.md, a GitHub App's
  delivery log; 17.6).
- **Checked:** the process tier's hinted story (13.4) checks the
  table's third row, CI on a head: the one wait of its landing that the
  forge ends. Its other waits end on temper's own writes, its push, its
  pull request and its merge, whose deliveries are echoes. Within its
  budget, only the job's end delivery brings a read of its CI:
  - the keep's `poll`, `poll_max` and `slow` are raised past the budget
    (configuration.md, section 7);
  - so is the change's `stall`, whose end reads its pull request afresh
    before it holds the change (`../domain/forge.md`, 5.3 and 8.4);
  - its job outlasts `settle` from its change's push, so the rereads
    within `settle` end before CI does (`../domain/forge.md`, 5.2).

  The first two rows, moved by someone else's push or edit, and the
  others, comments, reviews, issues and drift, are checked from hints
  in the worlds (`../domain/testing.md`, 2.2 and 6.3).

## 11. The crates

```
skein-forge                  + the setup's documents: admin users and their tokens, orgs, repositories'
│                              collaborators and hooks, GitHub's App documents; both sides, as its
│                              others; the pins, the versions and the fetched binaries' digests
│                              (skein's forge.md, sections 4 and 5)
skein-forge-setup            the setup's plans, their types and checks, the access they end with, and the
│                              host component a service runs them on (skein's forge-setup.md, section 6)
├── skein-forge
└── skein-lib, skein-json
temper-forge-setup           step code: the setup's types and checks, read from the configuration; the
│                              setup mapped onto the plans' given values, and the kit's check of them;
│                              the fetch's machine and the preparation's; app.ini and the runner's
│                              files; the processes' specs; the configuration's steps. Names no host
├── skein-forge-setup, skein-forge
└── jig-protocol-config, skein-lib, skein-json
temper-forge-setup-shell     ordinary Rust: the hosts of fetch, prepare and run: commands, files, the
│                              forge's locks, the exec, the fetch's requests over TLS, on skein-io
├── temper-forge-setup
└── skein-io, skein-http, skein-tls, skein-lib, skein-json, skein-shell
temper-box                   the box's machine (step code), its asked groups and its pipe to temper
│                              serve among it, and its host (ordinary Rust) on skein-io's processes;
│                              nothing of setup but its steps (7.3); one crate, under the subset's
│                              lints, its host written within the subset
├── temper-forge-setup
└── skein-lib, skein-io, skein-shell
temper-init                  step code: the default document and the files temper init writes, from
│                              the facts the binary finds (8.2, 8.3); reads, writes and binds nothing
└── skein-lib, skein-json
temper-fake                  step code: the demo's member, its types and checks; the fakes'
│                              configurations and seeds; the box's steps; the seed's plan (8.4).
│                              Names no host
├── temper-forge-setup
├── skein-fake-forge-domain, skein-fake-forge-protocol, skein-fake-llm-domain, skein-fake-peers
└── jig-protocol-config, skein-lib, skein-json
temper-fake-git              ordinary Rust: the fake git program's core, temper fake git, its replay and
│                              its two remotes (../domain/testing.md, 4.3)
├── skein-fake-checkout, skein-fake-forge-domain, skein-fake-forge-protocol, skein-fake-peers
└── skein-lib, skein-io, skein-http, skein-json
temper                       the binary: temper forge fetch|prepare|run, temper box, temper init,
                               temper fake llm|forge|git|seed
```

- **The engine runs the setup host** in its protocol layer (5.1):
  temper-service maps each forge's setup onto the plans' given values
  through `temper-forge-setup` (section 5), and depends on neither
  `temper-forge-setup-shell` nor `temper-box`.
- **The binary checks `setup` and its limits** for every command,
  `temper serve`'s included: `temper-forge-setup` checks `setup` and the
  `setup.` limits, and `temper-box` the `box.` limits, beside
  temper-service's startup.
- **Machines, not scripts.** The plans (skein's) and the preparation
  decide from what they read, and say each request or command they need;
  a host runs it and hands back its outcome. Step tests drive them with
  answers; the plans' world, skein's, drives them against the fake forge
  (13.2).
- **Not the throwaway's code.** `testing/skein-throwaway-forgejo` is a
  test crate (skein's `fake-forge.md`, section 6). The production
  machines are written on skein, under the subset, and the process tier
  provisions its Forgejo with them (13.4).
- **The fake forge serves the setup's documents** too
  (`skein-fake-forge-protocol`, skein's `fake-forge.md`, section 4),
  from the server's side of `skein-forge`, as it serves the engine's.
- **The fakes are product,** `crates/skein-fake-*` (skein's
  `fake-forge.md`, section 1), since `temper fake`
  ships them; the worlds and the real loop use them as before.

## 12. Where it serves

- **The first milestone** (`../README.md`, section 2), on a box run from
  a copy of the owned example (8.1), its CI Actions on the owned runner.
  `docs/guide/box.md` is its walk, from the binaries to a change asked
  for.
- **The process tier** (`../domain/testing.md`, 2.6), on owned mode in place of
  its own provisioning, with CI and webhooks (13.4).
- **QA's environments** (`../draft/qa.md`, 4.1), each an owned setup made
  from the same commands (7.4): a box an agent starts, drives through
  temper's client and Forgejo's API as the person, reads the logs in its
  files, and throws away.

## 13. Testing

The tiers, their budgets, the fakes and the layout are `../domain/testing.md`'s;
this section names what checks this document's contract, and what it
relies on.

### 13.1 Step tests

- **Forgejo's and the runner's files:** `app.ini` and the runner's
  configuration for each setup, as golden text.
- **The preparation's decisions:** a first preparation; one at the same
  versions; an upgrade, dumped before migrated; a newer recorded version
  refused; a runner added, under its lock, and one removed; another
  binary refused; a missing `admin_token` made again, under a fresh name;
  a preparation cut short made again from an empty layout; Forgejo's
  files with no `state` refused; a later preparation taking Forgejo's
  lock alone.
- **The specs:** each command's arguments and environment, and no secret
  of the preparation's in any argument.
- **The plans' decisions** from answers are skein's (skein's
  `forge-setup.md`, section 7). What is temper's is the mapping of
  section 5, in temper-forge-setup: each member of a managed setup and
  of an owned one becomes its value, the word `temper` and the names
  `temper-api` and `temper-git` among them, and each `Place` the kit's
  check refuses is named by its path.
- **The configuration:** each mismatch of section 8 refused, naming its
  path; a managed entry with a `directory`, and an owned or fake one
  without, refused; `steps` for each arrangement of modes, with and
  without the demo.
- **The demo,** in temper-fake: each bound of `application.demo`
  refused, naming its path; the fakes' configurations and seeds from
  the default document; the seed's plan, each act skipped when what it
  reads finds it done, and the link's second presenting last.
- **The box's machine:** the order; a restart after its once steps; the
  backoff and the cap; the stop's order, and `temper serve`'s second
  signal; a link step started on its long step's first line, given it
  alone, once per box, and given a restart's first line when the long
  step exited before writing one.
- **The fetch's machine** (6.6), in temper-forge-setup, given its URLs
  and digests: each pin's URL and digest for each architecture, and
  another architecture refused; another status or a redirect refused;
  a body past `setup.fetch_bytes` refused. With its host, in
  temper-forge-setup-shell's tests against a local file server over
  TLS, its root skein's test root named by `SSL_CERT_FILE`: a wrong
  digest, its part file removed and its refusal naming the URL and both
  digests; the right one, made 0o755 and renamed; a file already there
  hashed, nothing asked when it matches, and a mismatch refused, naming
  its path, the file left; a part file left by a fetch cut short
  written again.
- **The box's asked groups and its pipe** (7.1), in temper-box: a
  dormant group running nothing at the box's start; `start FORGE`
  running the group's steps in order, a `step` line for each, then the
  record written and `started FORGE`; a step's failure answering `failed
  FORGE WHY`, the group dormant again and the others running on; a
  second `start` while one runs answered busy, a re-ask of the starting
  forge answered by its own lines, an ask while stopping, a record not
  written, a forge with no group `not asked`, and a started one
  `started`; a start with the record
  running the group as an always group; a line past `box.line_bytes`
  refused; `temper serve` restarted with a pipe afresh, an answer meant
  for the exited one dropped.
- **The setup host's handover at runtime** (5.1), in temper-service's
  step tests: at `started`, `TakeOver`, then `Reconcile`, then the
  connect, the row "Connecting" between; a stop leaving the forge not
  connected, named; `failed` leaving it "Set up failed" with its reason;
  with no pipe, the row naming the commands (6.5).

### 13.2 Reconcile's world

The world is skein's (`forge-setup.md`, section 7): the plans on the
fake forge's protocol layer, serving the setup's routes, and the host
component's own waits and bounds. What is temper's is the engine's
routing of the setup host, checked in `temper-service`'s own tests
(`forge.md`, section 10) and by the process tier (13.4):

- a start's handover taken, its file removed only after `TakeOver`'s
  `Done`, and left when it stops; a handover readable by others
  refused, naming it;
- each write answered `Written` only once kept, so a restart between
  finds what a browser was told;
- a read served from the run's pasted value before the forge's record;
  a pasted value whose digest differs refused, "the pasted value
  changed";
- one writing plan per forge and one `Reachable` beside it, a second of
  either refused busy, and a request past `setup.runs` refused busy;
- a start's reconcile that stops leaving that forge not connected,
  named with its step and no secret, and the others served;
- records made for another kind or address leaving the forge not
  connected, named, nothing erased; an `api_token`, `git_token`, `user`
  or `webhook_secret` file in a forge's directory left unread.

### 13.3 What the setup relies on

Forgejo's and the runner's behaviours at the pinned versions (6.1), each
recorded by skein's conformance (`fake-forge.md`, section 7) on the
throwaway Forgejo and runner, and named here by its fact in the
conformance's `facts.txt`. What reconcile relies on, and the deliveries,
are skein's: `forge-setup.md`, section 8 and `forge.md`, section 9. What
the preparation relies on stays here:

- **Forgejo's command line** (6.3):
  - `admin user create --random-password` prints `generated random
    password is '<password>'` on its first line
    (`cli-user-create-prints-the-random-password-first`), and refuses a
    user made twice (`cli-user-create-again-is-refused-naming-the-user`);
  - `admin user generate-access-token --raw --scopes` prints the token
    alone (`cli-token-raw-prints-the-token-alone`), limited to the scopes
    it names, comma-joined (`cli-token-scopes-limit-what-the-token-reads`,
    `cli-token-admin-scopes-comma-joined-are-taken-under-two-suffixes`),
    and refuses a second token of one name
    (`cli-token-name-used-twice-is-refused`);
  - `migrate` runs on a fresh work path, and again on a migrated one
    (`cli-migrate-runs-on-a-fresh-work-path`,
    `cli-migrate-runs-again-on-a-migrated-work-path`);
  - `dump` needs the work path's repositories directory and its
    `--tempdir` (`cli-dump-needs-the-repositories-directory`,
    `cli-dump-needs-its-tempdir-to-exist`), adds `.zip` to a name without
    it (`cli-dump-appends-zip-to-a-file-name-without-it`), and overwrites
    a file of its name (`cli-dump-over-an-existing-file-overwrites-it`);
  - a user made to change its password is refused the API until it has
    (`cli-user-made-to-change-its-password-is-refused-the-api-until-cleared`).
- **The runner** (6.3):
  - `forgejo-cli actions register --secret-file` takes exactly 40
    hexadecimal digits with no line feed
    (`runner-register-refuses-a-secret-of-another-length`,
    `runner-register-refuses-a-secret-that-is-not-hexadecimal`), and run
    again with the same secret keeps one global runner
    (`runner-register-again-keeps-one-global-runner`);
  - the runner's `create-runner-file` takes its secret only as an
    argument
    (`runner-create-runner-file-takes-its-secret-only-as-an-argument`);
  - a connection of the runner's `server.connections` has, as its `uuid`,
    the secret's first sixteen characters in hexadecimal, and as its
    token the secret
    (`runner-uuid-is-the-secrets-first-sixteen-characters-in-hex`,
    `runner-token-is-the-secret`), which a file may hold (`token_url:
    file:`; `runner-connection-with-a-token-file-comes-online`); a runner
    started on that connection alone takes the instance's jobs
    (`runner-connection-alone-takes-the-jobs`).

### 13.4 The process tier

The tier, its budget and its referee are `../domain/testing.md`'s (2.6). What
it checks of this document:

- **Its directories:** every command a test runs is given the three
  directories' flags, each a scratch directory of the test's own; the
  owned example is copied as a configuration directory (8.1). No test
  resolves a default under `$HOME`.
- **Owned in place of the test's support:** the test runs `temper forge
  prepare`, `run forgejo` and `run runner` itself, then `temper serve`,
  whose setup host takes the handover and reconciles, with the binaries
  `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name, and webhooks on.
  Its repository runs Actions, so its landing rule asks for CI.
- **The hinted story:** the same story with the keep's `poll`,
  `poll_max` and `slow` and the change's `stall` raised past the story's
  budget (configuration.md, section 7; `../domain/forge.md`, 8.4), and its
  CI's job outlasting `settle` from the change's push, so that it lands
  within its budget only if the job's end is delivered and the reads
  that follow it (section 10, "Checked"); and with no conflict resolved,
  since its changes never conflict. With no webhooks, the same story
  misses its budget.
- **`temper box`,** a test of its own on a scratch directory: the sign-in
  link on its stdout; Forgejo killed and restarted, with the engine still
  serving; a stop leaving no process it started.
- **A first run,** a test of its own: `temper box` on three empty
  scratch directories writes the configuration (8.2), says so, and
  prints one sign-in link, which signs the owner in; the demo forge is
  connected, `demo/widget` adopted into project 1, and the project's
  chat answers a suggested request on the demo model (8.4); and the
  test's own clone of `demo/widget` by the demo's link, run as the
  worker runs git, holds the fake's landing head. A second box on the
  same directories does the same, its seed finding its acts done.
- **A first run set up for real,** a test of its own: `temper box` on
  three empty scratch directories, forge 3's `mirror` a local file
  server over TLS with skein's test root, serving at the pinned paths
  the binaries `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name, which
  the test checks against the pins first. The owner asks "Set up a
  Forgejo here" through the client; forge 3's row goes through "Setting
  up" and "Connecting" to connected, with no restart; the person's
  repository, made on Forgejo's API as the person, is adopted, and a
  worker clones it by the machine's git. A second box on the same
  directories starts the group at once, by its record, and fetches
  nothing.

## 15. Later

Each is intended; what it needs is named with it.

- **A lock and an exec in skein's shell,** onto which 6.4's module
  moves once skein's shell offers an exec and a lock held across it,
  which skein's `Lock` is not (skein's `io.md`, section 5). A second
  service that execs is what would ask skein for them.
- **Reconcile over TLS,** managed mode against a forge on another host,
  for a deployment whose managed forge is not on loopback (skein's
  `forge-setup.md`, section 9).
- **Webhooks from another host,** over TLS (skein's
  `forge-connection.md`, section 6), once skein has TLS's server side.
- **CI's isolation.** An owned runner's jobs in host mode run as the
  box's user, who can read every file of temper's as credentials.md,
  section 8 accepts of agents in standalone, the handover among them
  until the engine takes it. Until they are isolated, an owned forge
  runs the person's and temper's agents' code only, never a stranger's
  pull request.
- **Provisioning at adoption** (jig's `domain/connectors.md`, 12): a
  repository in a person's own namespace, made without the person.
- **The forge as people's sign-in provider** (credentials.md, section
  8), which would link the person's account on an owned Forgejo to their
  owner in jig.

## 16. Open questions

- **Good defaults for a person's real work:** charters, landing rules,
  limits, the memory bound. Default: 8.3's working minimum, refined once
  a first run works.
- **Overrides of `app.ini`.** Default: none; the settings are temper's.
- **A token for the person's scripts and agents.** Default: none; the
  person makes one on Forgejo's web, with the scopes they choose. A QA
  harness playing the person makes one so, with read scopes alone, for
  the agent it runs (`../client/tools.md`, section 10); never the
  preparation's `admin_token`, which the engine's setup host alone takes
  (section 9).
- **Backups beyond an upgrade's.** Default: the person's, of the forge's
  directory and the store, with the box stopped.
- **Two deployments on one managed forge.** Default: each its own bot
  login, and so its own collaborators and hooks; the marker does not
  tell deployments apart.
- **A hook left at an old listener's address,** after the listener
  moves, is skein's question (skein's `forge-setup.md`, section 9).
  Default: left alone; the person deletes it.
- **A managed forge whose host list refuses the listener.** Default:
  nothing tells the person but the hints that never arrive
  (`../domain/forge.md`, section 4).
- **The managed picker's departure.** On a managed or owned Forgejo the
  picker lists every repository the site admin can see, not those the
  bot can write, as a connected forge's does, since adoption grants the
  bot its access (5.4). Default: the admin's view, through `Reachable`,
  plus "not listed" by owner and name; the admin's token stays the
  setup host's, and the picker gets the rows alone.
- **"Connect as its admin" on a Forgejo with no `setup` entry.**
  Default: refused, naming `application.setup`, since the bot's login
  and email are the configuration's (5.5).
- **A start's reconcile that stops.** Default: that forge is not
  connected, named with its step, until the next start or a managed
  connect; the engine runs no reconcile again within a process.
- **A managed or owned forge stopped at runtime,** its bot's token
  revoked. Default: stopped, named, until the next start's reconcile or
  a managed connect; Replace is a connected forge's alone.
- **The owned example's agent, sized for a person's work.** Default: the
  process tier's profile (8.1), sized for the live tier's stories.
- **The owned example's `memory_bytes`** are `small`'s worst cases with
  its strings at their bounds (8.1): 78,669,036,604,974 bytes for the
  engine and 1,989,738,112,203 for the worker, more than any machine
  has, so the start's check bounds nothing a person's machine holds.
  Default: as the example states them; which terms of the worst cases
  make them so large is open (configuration.md, section 11).
- **The check's decisions** (8.4) are read from rows kept for other
  ends, and two decisions leave none: a change whose pull is replaced
  while `Landing` joins its `Landed` row to the later pull, and an
  enqueue the forge's merge queue drops leaves no `Entry` once settled,
  a GitHub path the Forgejo fake never takes. Default: as 8.4 reads
  them; a family of temper's decisions, `(repository, pull, head)`
  written in the merge's `Entry`'s commit, is the route when a run
  needs either, a store version's change (store.md, section 4).

## 17. Connecting GitHub

GitHub is managed or connected, never owned. Both modes end in the same
records (section 3), and the engine runs both alike after; they differ
in who made the App. The plans are skein's (`forge-setup.md`, section
5), run by the setup host (5.1); the browser's routes are the clients'
listener's (`../client/forge.md`, the browser's steps).

### 17.1 GitHub's App and its permissions

- **One App per deployment,** public, so that it installs on any
  account (skein's `forge-setup.md`, section 5). Its permissions:
  contents, pull requests, issues and merge queues, write; checks,
  statuses, actions, administration and metadata, read; never
  workflows, never administration's write. Its events are skein's
  (5.6).
- **Its hook** points at `https://<slug>.invalid/`, an address that
  never resolves, so GitHub delivers nowhere and holds no secret of
  temper's; the App's changes come from its delivery log (17.6).
- **Its records:** `AppId`, `AppSlug`, `AppKey` (DER; a PEM is never
  kept), `User` (the App's bot user), `Accepted` and `Origin`
  (credentials.md, the records).
- **Its addresses** are github.com's when the forge's `api`, `git` and
  `web` are null (configuration.md, section 3); a forge names them only
  to point at a fake.
- **The clients' listener at a fixed address,** a loopback port the
  person's browser reaches, since GitHub's redirect and setup URLs name
  it; its port 0 is refused at start while a GitHub forge is
  configured.
- **A repository's own rules** are read and said at adoption, and never
  changed (`../domain/forge.md`, what adoption says of the branch's
  rules): a merge queue is taken as it is; required approvals need the
  App on the rule's bypass list or a person's approval; required
  signatures, linear history and code owners are each named with the
  one setting that lets temper land.

### 17.2 Managed: the manifest flow

1. **Connect,** "Connect GitHub" on the deployment's page or its tool
   of `temper ctl`, for the person's account or an org they name
   (`Connect` with `GitHubManifest { org? }`). The setup host runs
   `Manifest { forge, target }` (skein's `forge-setup.md`, 5.1) for a
   fresh state, ending `Done(Manifest { state, page })`; the page is
   held with the state. The client shows the
   link, `http://<listener>/forges/<n>/github/manifest?state=<s>`, as
   text, and never opens it.
2. **The manifest's page,** opened by the person: one form posting the
   manifest to `<web>/settings/apps/new?state=<s>`, or
   `<web>/organizations/<org>/settings/apps/new?state=<s>`, one button,
   no script; its headers allow a form to `<web>` alone, keep no copy
   and send no referrer.
3. **GitHub's form:** the person creates the App.
4. **The redirect back,** `http://<listener>/forges/<n>/github/redirect`:
   `Convert { forge, code, state }` (5.2) exchanges the code for the
   App's id, slug and key, finds its bot user, and writes the records;
   the conversion's PEM, webhook secret and OAuth pair are never kept.
   Once they are kept, `Install` (5.4) answers the App's install link,
   the browser goes on to it, and the forge connects (forge.md, the
   setup's host).
5. **Installing:** the person picks an account, and all or selected
   repositories; temper's client advises selected. GitHub sends the
   browser to `http://<listener>/forges/<n>/github/setup`, where
   `Accept` (5.4) checks the installation and adds it to `Accepted`;
   the page says "temper has the installation; you may close this
   tab."

- **States** are one use, drawn by the setup host, at most
  `setup.states`, each living `setup.state_life_ms`, at most the
  conversion code's hour; a used, unknown or expired one is refused,
  "This link has ended. Start again from temper.", and a restart
  forgets them all.
- **The redirect and setup URLs** temper gives the plans are whole,
  built from the clients' listener's address (forge.md, the setup's
  host). The manifest's `url`, the App's homepage, is
  `http://<listener>/`.

### 17.3 Connected: an App pasted

- **The person makes an App** on GitHub with 17.1's permissions and
  events, and pastes its id and its private key as GitHub gives it,
  PEM: "Connect an App of your own" (`Connect` with `GitHubApp { id, key
  }`). No webhook secret is asked: the paste check sets the App's hook.
- **Its setup URL is the person's to give** on GitHub's form, since no
  plan can set it: `http://<listener>/forges/<n>/github/setup`, with
  redirect on update, no callback URL and no user's authorization at
  install (`docs/guide/github.md`). An installation of an App without
  it never comes back through the setup route, and is a stray (17.4).
- **The paste check,** `Paste { forge, app }` (skein's
  `forge-setup.md`, 5.8): the key, kept by jig until its run ends,
  decoded to DER; the App read with it, its id the one pasted, else
  `OtherApp`; its bot user found; its hook set to
  `https://<slug>.invalid/`; its permissions and events checked as its
  reconcile checks them. It writes `AppId`, `AppSlug`, `AppKey` and
  `User`, the records the manifest flow writes, and ends done, or
  `Lacking` with each permission or event named, never refused for it.
- **Installing** is 17.4's.
- **Replace,** a new key: the paste check with the kept id, so a key of
  another App stops it `OtherApp` and the forge stays as it was.

### 17.4 Another account, and repositories later

- **"Install on another account"** (`Install { forge }`, skein's
  `forge-setup.md`, 5.4): a fresh state and GitHub's install link
  (`Link`), shown as text; the person installs on another account or
  org, and the setup route accepts it.
- **Repositories added later** to an installation of selected
  repositories are the person's, on the installation's page on GitHub;
  GitHub's visit to the setup route with no state refreshes an accepted
  installation's repositories, and is refused otherwise. The forge
  component reads an installation's repositories again on its events.
- **An installation temper did not accept,** a stray made with no
  state of temper's, is shown, and uninstalled by the next reconcile
  once older than a state's life; one deleted on GitHub leaves
  `Accepted`; one suspended is shown.

### 17.5 Its reconcile

- **`Reconcile { forge }`** (skein's `forge-setup.md`, 5.6) runs after
  each `Connected`: the App's key, permissions, events and installations
  checked. It repairs only its hook's address.
- **A key GitHub refuses** stops the forge: the forge component, which
  signs with the same key, stops it (forge.md, 2.3), named, with
  Replace (17.3); the reconcile's own `Stop` is named to the root and
  changes nothing.
- **A permission or event lacking** is named, `Lacking`, never refused:
  the person grants it in the App's settings and approves it on each
  installation, as GitHub asks.

### 17.6 Job logs, and the delivery log

- **A job's log** lies behind a redirect to a storage host, which the
  forge component follows only to an origin its `log_origins` lists
  (configuration.md, section 3; forge.md, GitHub's API as temper uses
  it): on GitHub one origin by default; `[]` fetches no job log. An
  origin not listed is named on the deployment's page, and the person
  adds it to the forge's `log_origins` and restarts.
- **The delivery log** is read by the forge component every
  `log_poll`, and its deliveries become hints as Forgejo's webhooks do
  (forge.md, a GitHub App's delivery log).
