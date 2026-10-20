# The forge's setup

Provisional, 2026-10-16, revised 2026-10-20. How a forge comes to be
ready for temper, in three modes: connected, where a person runs and sets
up the forge and gives temper only what it strictly needs; managed,
where a person runs it and temper sets itself up on it, with a Forgejo
site admin's token or, on GitHub, with an App made from temper's
manifest; and owned, where temper runs Forgejo and its runner, and sets
itself up on them. A person chooses a forge's mode and connects it from
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
  browser. Owned: temper runs Forgejo and its runner.
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
  are machines and pure functions of `temper-forge-setup`, hosted by two
  commands, `temper forge prepare` and `temper forge run` (section 6).
  No command writes the records.
- **`temper box` is wiring only.** It runs a configuration's steps in
  order (prepare, Forgejo, its runner, `temper serve`), restarts what
  exits, and stops them in reverse order. It names no Forgejo request
  and nothing of setup but its steps. systemd, QA's environments or a
  test arrange the same commands another way (section 7).
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
  conformance's (section 6). Its directory, once temper stops running
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
- **Not here:** binaries' digests, webhooks from another host, the
  isolation of CI's jobs, provisioning at runtime and the forge as
  people's sign-in provider (section 15).

## 2. The three modes

| | connected | managed | owned |
|---|---|---|---|
| Runs the forge and CI's runner | the person | the person | temper |
| Makes temper's bot, its tokens, its access and the webhooks | the person | temper: on Forgejo with a site admin's token; on GitHub, an App from temper's manifest | temper, with an admin it makes |
| temper is given, on Forgejo | the bot's two tokens, pasted; optionally a webhooks' secret | a site admin's token, pasted | the Forgejo and runner binaries; an address on loopback; the person's login, email and orgs |
| temper is given, on GitHub | an App's id and its private key, pasted | the person's steps in their browser, from a link temper prints | none: GitHub is never owned |

- **Per forge.** A deployment may mix them: an owned Forgejo beside a
  connected one, or beside GitHub. Each owned forge has a directory and
  an address of its own.
- **A mode is no record** (credentials.md, the records): a forge is
  owned when its `setup` entry says so (section 8); managed when it is
  not owned and its `AdminToken` record is kept; connected otherwise. On
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
`forge-setup.md`, section 3. temper's layers, and what passes between
them:

```
owned      temper forge prepare      ──► the forge's work path, the runner's, person_password,
                                         admin_token (the handover)
           temper forge run          ──► forgejo web, forgejo-runner daemon (each exec'd)
managed    the engine's setup host   ──► the forge's records (below); takes the handover once
connected  the forge component       ◄── its access, built from the records
```

- **One directory per owned forge,** named by its setup (section 8):
  private, `0o700`, the user's own and not reached through a link, as a
  worker's root is (configuration.md, section 4). `temper forge
  prepare` makes it so when it is absent; a directory that is not so
  refuses the command, naming it. A managed or connected forge has
  none. Its files:

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
  | Forgejo, managed or owned | those, and `AdminToken`; `WebhookSecret` drawn by reconcile when the webhooks' listener is configured |
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
of a managed or owned Forgejo and at a managed connect. No command runs
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
- **At each start,** once the records are loaded: an owned forge's
  handover, when its file exists, by `TakeOver` (6.5); then `Reconcile`
  for each managed and owned Forgejo; and GitHub's `Reconcile` after each
  GitHub forge connects (17.5). The deployment's page shows a forge
  "Connecting" until its plan is done and it connects.
- **A run's end:** `Done`, or `Lacking` (done, with what the App lacks
  named), keeps its writes, then connects the forge after a plan that
  connects; `Stop` names its failure and its step, with no secret, and
  leaves the forge as it was. A start's reconcile that stops leaves that
  forge not connected, named, until the next start or a managed connect;
  the engine and every other forge run on.

### 5.2 The setup host's plans

| Plan | Forge | When | skein's `forge-setup.md` |
|---|---|---|---|
| `Reconcile { forge, adopted }` | Forgejo, managed or owned | each start; a managed connect | section 4 |
| `Adopt { forge, repository }` | Forgejo, managed or owned | an adoption, before its reads | 4.5 |
| `Check { forge }` | Forgejo, connected | a connect; a replacement | 4.6 |
| `TakeOver { forge, admin_token }` | Forgejo, owned | a start that finds the handover | 4.7 |
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
  there stops it `Installation`, `Unexpected`. On a managed or owned
  Forgejo it reads every repository the admin can see, by id ascending,
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
  | the orgs, and their owner | `mode.owned.person.orgs` and `mode.owned.person.login` when owned (6.5); none when managed |
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
  managed or owned Forgejo (skein's `forge-setup.md`, 4.5): the bot made
  a collaborator with write, and the repository's hook made when the
  listener is configured, with the forge's secret. Stopped, the adoption
  is refused, named (`../domain/forge.md`, access at adoption).
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

- **Inputs:** the Forgejo binary's absolute path, and the runner's, or
  no runner; the address Forgejo listens on, a socket address on
  loopback; the person's login, email and orgs; the runner's labels,
  capacity and `PATH`; the forge's directory.
- **The binaries are checked** by every command that runs them: the
  first line of each one's version equal to temper's pin, Forgejo's and
  the runner's (`skein-forge`'s `VERSION` and `RUNNER_VERSION`, skein's
  `forge.md`, section 4.3), the versions skein's conformance checks
  (`fake-forge.md`, section 7). Another binary refuses the command,
  naming its path and what it printed. temper downloads nothing.
- **One pin,** `skein-forge`'s, read by owned mode, the throwaway and
  the conformance alike, so the Forgejo temper runs is the Forgejo whose
  facts it checked (13.3).

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

`temper forge prepare CONFIG.json FORGE`, before Forgejo runs. It takes
Forgejo's lock (6.4), and the runner's too when it writes the runner's
files: at a first preparation, and for a runner older than its pin
(item 3) or added (item 4); checks the binaries; and decides by
`state`:

1. **The first preparation:** the layout of section 3; `app.ini`;
   `forgejo migrate`; temper's admin, `temper-admin`, made as a site
   admin with a password Forgejo draws and nobody keeps, and its token
   made by `admin user generate-access-token` into `admin_token`, the
   handover (section 3), with the scopes `write:admin`,
   `write:organization` and `write:user` (the scopes the conformance
   records reconcile's routes under; skein's `forge-setup.md`, section
   8, says which route needs which), under a name that ends in a random
   suffix, since Forgejo refuses a second token of one name; the
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

`temper forge run CONFIG.json FORGE forgejo|runner`:

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

- **The handover,** at `temper serve`'s start: the setup host finds
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
  record, is not connected, and the client says "prepare it with
  `temper forge prepare`" (`../client/forge.md`, the deployment's page).
- **The person** signs in on Forgejo's web as a site admin with
  `person_password`, and changes it, since Forgejo's API refuses them
  until they do (13.3); makes repositories in their orgs, with Actions,
  and adopts them through temper's client. temper's admin is temper's; a
  person who deletes it makes the next start's reconcile and the next
  preparation stop, naming it.
- **The forge's members** are connected's, written out: `api` the
  address, `transport` plaintext, `git` `http://<address>` (section 8).

## 7. The box: wiring only

### 7.1 What it runs

`temper box CONFIG.json` runs the configuration's steps, which
`temper-forge-setup`'s `steps` gives, each a command of the `temper`
binary, as a child process:

```
temper box CONFIG.json
├── temper forge prepare CONFIG.json 2        once, per owned forge
├── forgejo web                               long: temper forge run CONFIG.json 2 forgejo
├── forgejo-runner daemon                     long: temper forge run CONFIG.json 2 runner
└── temper serve CONFIG.json                  long: its own tree (README.md, section 4); its setup
                                              host takes each handover and reconciles (section 5)
```

### 7.2 What it does

- **In order:** each step starts once the one before it has started, when
  that one is long, or has exited 0, when it is once.
- **Restarts** a long step that exits, after a backoff doubling from
  `box.backoff_ms` to `box.backoff_max_ms` (configuration.md, section 7),
  first running again the steps that are once just before it: a Forgejo
  is restarted after its preparation, so a change of the setup reaches
  its `app.ini`. `temper serve` restarts on its own, and its start takes
  a handover the preparation left and reconciles each managed and owned
  forge. A token revoked under a running engine stops that forge alone,
  named, and not the engine (forge.md, a forge stopped); the engine's
  next start makes it again.
- **Gives up,** exit 1, naming the step, when a once step fails or a long
  one exits more than `box.restarts` times within `box.window_ms`. The
  backoff is counted from a step's exits within the window, so it starts
  again from `box.backoff_ms` once a step has run a window without
  exiting.
- **Stops** at a first termination signal: the long steps in reverse
  order, each by a signal, `box.grace_ms`, a second signal, which turns
  `temper serve`'s drain into an abort (README.md, section 4),
  `box.grace_ms` again, then a close. The machine knows a step only as
  once or long, so every long step is sent both. A second termination
  signal closes them all. It exits 0 after a stop it was asked for.
- **Its streams:** `temper serve`'s stdout, the sign-in links, is the
  box's stdout, as is. Every other line a step writes, to its stdout or
  its stderr, goes to the box's stderr prefixed `<step>: `; the box's own
  lines are prefixed `box: `, such as `box: <step> gave up after <n>
  restarts`. Each step gets the box's environment; `temper forge run`
  clears it for Forgejo (6.4). Forgejo logs to its own files.
- **Its children** are skein-io's (skein's `io.md`, section 6), each in a
  process group of its own; closing one kills its group, so nothing the
  box started outlives its stop.

### 7.3 What it names

- **Steps only:** each step's name, its arguments and whether it is once
  or long. Its policy is a small machine of its own (steps, exits and
  timers in; spawns, signals and closes out), hosted on skein-io.
- **The binary hands it the steps.** `temper box` checks the document as
  section 8 says, asks `temper-forge-setup` for its steps, and gives them
  to `temper-box`.
- **Checked:** a boundary test fails the gate when `temper-box` depends
  on a crate named `temper-*` but `temper-forge-setup`, or names an item
  of it but `steps` and `Step`.

### 7.4 Other arrangements of the same commands

Each needs nothing below the commands to change:

- **systemd,** a unit per long step: `prepare` as `ExecStartPre=` of
  Forgejo's unit, `run` and `temper serve` as `ExecStart=`, the order as
  the units' dependencies; `temper serve`'s unit keeps systemd's default
  kill of its control group, which ends a crashed service's whole tree
  before its restart (skein's `shell.md`, 13);
- **the process tier** (section 13.4), which runs the commands itself;
- **QA's environments** (`../draft/qa.md`, 4.1), whose "temper in a box" is
  an owned setup with a scripted LLM, made by its own host from the same
  commands, its LLM key added from the client once signed in
  (`../client/tools.md`, section 10);
- **a container image** whose entry point is `temper box`, every process
  of its tree ending with the container.

## 8. The configuration

- **One document,** `temper serve`'s (configuration.md, section 2).
  Every `temper forge` command and `temper box` read it whole, check it
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
                  "orgs": ["work"]}}}}
  ]
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `forge` | u16 | the `number` of one entry of `forges`, of kind Forgejo |
  | `directory` | a path | when owned, and only then: a private directory (section 3), unique among `setup` |
  | `bot` | `{"login", "email"}` | a Forgejo login; an email, the bot's when reconcile makes it (5.6) |
  | `mode` | `"managed"` or `{"owned": ...}` | managed: temper may be given a site admin's token for the forge from the client (5.5) |
  | `owned.forgejo`, `owned.runner.binary` | absolute paths | checked against the pins of 6.1 |
  | `owned.runner` | the runner's members, or null for none | `capacity` at least 1; `labels` at least one, at most 8 |
  | `owned.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and owned forges |
  | `owned.person` | `{"login", "email", "orgs"}` | a Forgejo login other than the bot's and `temper-admin`; `orgs` at least one, at most `setup.orgs`, each a Forgejo org name |

- **A Forgejo with no `setup` entry** is connected's alone; GitHub takes
  no entry (section 17).
- **The forge's own members name no credential:** its records hold
  them (section 3). They stay connected's, written out, never derived,
  and checked against its setup; a mismatch refuses every command,
  naming the path: when owned, its `api` is `listen`, its `transport`
  plaintext and its `git` `http://<listen>`; an identity's `login`, when
  given, is the bot's.
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
  forge's handover (section 3).
- **An owned example** ships for a person, a test or QA's host to start
  from (8.1).

### 8.1 The owned example

`docs/examples/owned.json` is a whole document, `engine`, `workers` and
`application`, for one owned forge with a runner, webhooks, one git
identity, one worker and two LLM providers, with no LLM account: a
person adds one from the client once the deployment has started
(credentials.md, section 3). The files it names that hold no secret are
beside it, in `docs/examples/owned/`: the agent's configuration and the
charters, short texts a person edits. The files that hold a secret, the
worker's secret and its digest, are never shipped: `docs/guide/box.md`
says how each is made. A person, a test or QA's host
starts from a copy, and `temper box CONFIG.json` runs it (section 7).

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
  whole: 1, OpenAI, with ChatGPT's sign-in, its registration public and
  its redirect's port ephemeral, and keys; and 2, Anthropic, with Claude
  Code as a program, and keys. Each with its endpoint, its models and
  their prices. The charters allow each provider's models in order, the
  default the first whose LLM provider has a usable account (llm.md,
  section 5; jig's `domain/tasks.md`, section 3).
- **Its agent's configuration** (configuration.md, section 5),
  `owned/agent.json`, declares both providers' endpoints and models, and
  Claude Code's program, its path and its programs directory, a
  directory the person names; and takes the process tier's profile, the
  sizes its live tier runs a real model with: smith's `standard`, with
  `declared` `memory` 2,147,483,648, `tool_payload` 600, `read_window`
  256, `conversations` 1 and `calls_per_response` 1, and `policy`
  `max_turns` 5, `group_stop` 10, `inbox` 8 and `unacknowledged` 8. Its
  messages are then 600 bytes, and its largest call 4,096 bytes, its
  `outcome_bytes` (configuration.md, section 7, `small` against the
  agent). smith's standard sizes with nothing declared refuse the
  example's model declaration at start.
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
  has 8; and `hub.event_bytes` 598, the agent's `message_bytes`, 600,
  less smith's 2-byte separator, where jig's small has 65,536. Its
  `application.limits` is `small` with no override.
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

## 9. Credentials

| Credential | Mode | Made by | Kept | Used by |
|---|---|---|---|---|
| a site admin's token | Forgejo, managed; owned | the person, pasted; the preparation, by Forgejo's command line | the `AdminToken` record; an owned forge's handover, until the engine takes it | the setup host alone, while a plan runs: reconcile, `TakeOver`, access at adoption (`Adopt`), and the picker's read (`Reachable`, skein's `forge-setup.md`, 4.8) |
| the bot's API and git tokens | Forgejo, every mode | the person, pasted; reconcile | the `ApiToken` and `GitToken` records | the forge component; the identities' table |
| the bot's password | Forgejo, managed, owned | reconcile, random, when it makes the bot | nowhere | nothing |
| temper's admin's password | owned | Forgejo, random | nowhere | nothing |
| the person's first password | owned | Forgejo, random, at the first preparation | `person_password` | the person |
| the runner's secret | owned | the preparation | `runner/secret` | Forgejo's command line; the runner's connection, through the token's file it names |
| Forgejo's own secrets | owned | the preparation | `state`, and `app.ini` | Forgejo |
| the webhooks' secret | Forgejo, every mode | the person, pasted; reconcile, drawn | the `WebhookSecret` record | the webhooks' listener; Forgejo's hooks |
| a GitHub App's key | GitHub | GitHub, at the manifest's conversion; the person, pasted as PEM | the `AppKey` record, DER; a PEM never | the forge component, for its tokens; the setup host, while a plan runs |
| an installation's token; a GitHub git token | GitHub | the forge component, from the App's key | memory alone | the forge component; git |

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
│                              others; the pins (skein's forge.md, sections 4 and 5)
skein-forge-setup            the setup's plans, their types and checks, the access they end with, and the
│                              host component a service runs them on (skein's forge-setup.md, section 6)
├── skein-forge
└── skein-lib, skein-json
temper-forge-setup           step code: the setup's types and checks, read from the configuration; the
│                              setup mapped onto the plans' given values, and the kit's check of them;
│                              the preparation's machine; app.ini and the runner's files; the processes'
│                              specs; the configuration's steps. Names no host
├── skein-forge-setup, skein-forge
└── jig-protocol-config, skein-lib, skein-json
temper-forge-setup-shell     ordinary Rust: the hosts of prepare and run: commands, files, the forge's
│                              locks, the exec, on skein-io
├── temper-forge-setup
└── skein-io, skein-lib, skein-json, skein-shell
temper-box                   the box's machine (step code) and its host (ordinary Rust) on skein-io's
│                              processes; nothing of setup but its steps (7.3); one crate, under the
│                              subset's lints, its host written within the subset
├── temper-forge-setup
└── skein-lib, skein-io, skein-shell
temper                       the binary: temper forge prepare|run, temper box
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
  (`testing/skein-fake-forge-protocol`, skein's `fake-forge.md`, section
  4), from the server's side of `skein-forge`, as it serves the
  engine's.

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
  path; a managed entry with a `directory`, and an owned one without,
  refused; `steps` for each arrangement of modes.
- **The box's machine:** the order; a restart after its once steps; the
  backoff and the cap; the stop's order, and `temper serve`'s second
  signal.

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

## 15. Later

Each is intended; what it needs is named with it.

- **A lock and an exec in skein's shell,** onto which 6.4's module
  moves once skein's shell offers an exec and a lock held across it,
  which skein's `Lock` is not (skein's `io.md`, section 5). A second
  service that execs is what would ask skein for them.
- **The binaries' digests,** an optional SHA-256 per binary beside the
  version's check, for a deployment that installs binaries it did not
  build or download itself.
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

- **A command that writes the document** (`temper box init`). Default:
  the owned example only.
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
- **The scopes of a managed forge's admin's token.** Default: the
  preparation's, `write:admin`, `write:organization` and `write:user`
  (6.3), as `docs/guide/forgejo.md` asks; skein's facts (`forge-setup.md`, section 8)
  name `write:admin` for the token routes and a user scope for `GET
  /user`, and none for the repositories' routes `Adopt` calls.
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
