# The forge's setup

Provisional, 2026-10-16. How a forge comes to be ready for temper, in
three modes: connected, where a person runs and sets up Forgejo and gives
temper only what it strictly needs; managed, where a person runs it and
gives temper a site admin's token, with which temper sets itself up; and
owned, where temper runs Forgejo and its runner, and sets itself up on
them. The engine is the same in every mode: each mode makes the files the
one below reads, and the engine reads connected's. Owned mode is commands
any host can arrange; `temper box` is one arrangement, and wiring only.
A box is what temper's first milestone is set up on and checked by CI on
(`draft/mvp.md`), and what QA's agents drive (`draft/qa.md`, 4.1). The
members are configuration.md's (section 3), the credentials
credentials.md's (section 2), the webhooks forge.md's (section 5), and
the commands and crates protocol.md's (sections 4 and 5). What comes
later is section 15; what is open, section 16.

## 1. In one page

- **Three modes, per forge** (section 2). Connected: a person runs and
  sets up Forgejo, and temper is given its bot's tokens and id. Managed:
  a person runs it, and temper is given a site admin's token. Owned:
  temper runs Forgejo and its runner.
- **Each mode is a layer that makes the inputs of the one below.**
  Owned's preparation makes a Forgejo and an admin's token, managed's
  input; managed's reconcile makes the bot, its tokens, its id and the
  webhooks' secret, connected's inputs. The engine always runs
  connected: it reads the same files, checks the same user, and never
  holds an admin's token (section 3).
- **Files are the interface.** Each layer writes private files in the
  forge's directory, under fixed names, and the next reads them. No layer
  calls another, so any host can run them in any arrangement.
- **The core is step code; the hosts are thin.** Reconcile, the
  preparation, Forgejo's settings and the processes' specs are machines
  and pure functions of `temper-forge-setup`; running a command, a
  request or a file write is a host's. Three commands host them: `temper
  forge prepare`, `temper forge run` and `temper forge reconcile`
  (sections 5, 6 and 11).
- **`temper box` is wiring only.** It runs a configuration's steps in
  order (prepare, Forgejo, its runner, reconcile, `temper serve`),
  restarts what exits, and stops them in reverse order. It names no
  Forgejo request and nothing of setup but its steps. systemd, QA's
  environments or a test arrange the same commands another way
  (section 7).
- **Reconcile converges at every start:** the bot user, its two tokens,
  a team with write on every repository of each configured org, the bot
  in it, and each org's webhook, each made when missing, repaired when
  changed and left alone otherwise. The admin's token is read then, by
  `temper forge reconcile` alone (section 5).
- **Owned is a stock Forgejo.** temper writes its `app.ini`, runs its
  migrations, makes its own admin and the person's account with
  Forgejo's command line, registers the runner offline, dumps before an
  upgrade's migration, and runs only the versions it pins, the
  conformance's (section 6). Its directory, once temper stops running
  it, is a Forgejo a person can run themselves, as managed.
- **A box is reactive.** Every change on the forge that a box's work
  waits on reaches the engine as a signed webhook delivery, and becomes a
  hint, read within the keep's `hinted`: pushes, pull requests, issues, comments,
  reviews and each CI job's end. Reconcile makes the hooks; an owned
  Forgejo is allowed to deliver to the engine's listener. Polling stays
  as it is, the fallback for a delivery lost or refused (section 10).
- **Loopback.** An owned Forgejo listens on loopback, as temper's
  clients' and webhooks' listeners do; a person on another machine
  reaches them through a tunnel. A forge on another host polls until
  skein's TLS server side.
- **Not here:** GitHub, binaries' digests, webhooks from another host,
  the isolation of CI's jobs, provisioning at runtime and the forge as
  people's sign-in provider (section 15).

## 2. The three modes

| | connected | managed | owned |
|---|---|---|---|
| Runs Forgejo and CI's runner | the person | the person | temper |
| Makes temper's bot, its tokens, its access and the webhooks | the person | temper, with a site admin's token | temper, with an admin it makes |
| temper is given | the bot's two tokens and its id; optionally a webhooks' secret | a site admin's token; the orgs temper works in | the Forgejo and runner binaries; an address on loopback; the person's login; the orgs |

- **Per forge.** A deployment may mix them: an owned Forgejo beside a
  connected one. Each set-up forge has a directory of its own and, when
  owned, an address of its own.
- **A mode is changed by the configuration and a restart:**
  - owned to managed: temper stops running Forgejo; its directory's
    `forgejo/` is a stock work path, and its `admin_token` stays valid;
  - managed to connected: the forge's entry of `setup` is removed; no
    reconcile runs, and its files stay as it last wrote them;
  - connected to managed: an admin's token is given, and reconcile takes
    over a bot it made before, or makes one (section 5).

## 3. The layers and their files

```
owned      temper forge prepare     ──► the forge's work path, the runner's, admin_token, person_password
           temper forge run         ──► forgejo web, forgejo-runner daemon (each exec'd)
managed    temper forge reconcile   ──► api_token, git_token, user, webhook_secret
connected  temper serve             ◄── reads them, through the forges and identities that name them
```

- **One directory per set-up forge,** named by its setup (section 8):
  private, `0o700`, the user's own, and refused otherwise, as a worker's
  root is (configuration.md, section 4). Its files:

  | File | Made by | Read by | Holds |
  |---|---|---|---|
  | `forgejo/` | prepare | Forgejo, through `run` | Forgejo's work path: `app.ini`, its database, repositories and logs |
  | `runner/` | prepare | the runner, through `run` | the runner's configuration, its registration, its jobs' directories, `secret` |
  | `backups/` | prepare | the person | one dump before each upgrade's migration |
  | `state` | prepare | prepare, `run` | owned's record: the versions prepared, whether the first preparation ended, Forgejo's own secrets |
  | `admin_token` | prepare, when owned | reconcile | temper's admin's token |
  | `person_password` | prepare | the person | the person's first password on the owned Forgejo |
  | `api_token` | reconcile | the engine's forge component | the bot's API token |
  | `git_token` | reconcile | the identities' table | the bot's git token |
  | `user` | reconcile | the engine | the bot's id, in decimal |
  | `webhook_secret` | reconcile | the engine's webhooks' listener | the secret of the hooks reconcile makes |

  A managed forge's admin's token is the person's file, named by its
  setup, wherever it is.
- **Written whole.** Each file is written beside its name and renamed
  over it, private, so a reader never sees half a token.
- **Read as configuration.md, section 6 reads a secret's file,** each
  within its bound.
- **Nothing passes between the layers** in an argument or the
  environment but the configuration's path and the forge's number.

## 4. Connected

- **What temper strictly needs,** for each forge: the bot's API token,
  its git token and its id (credentials.md, sections 4 and 5); for
  webhooks, their secret. The scopes, checked on v17.0 (section 13.3):
  `read:user`, `write:issue` and `write:repository` for the API token,
  `write:repository` alone for the git token.
- **What the person does:** a Forgejo user for temper, not a site admin;
  its two tokens; its write access to each repository temper will adopt,
  as a collaborator or through a team; optionally, each repository's or
  org's webhook to temper's listener, with the secret and the events of
  forge.md, section 5, and Forgejo's `[webhook] ALLOWED_HOST_LIST`
  letting it reach the listener; CI, as each repository has it.
- **Nothing more is required** (`domain/forge.md`, section 4): polling
  stands in for webhooks, and a project's checks for CI.
- **The engine as configuration.md has it,** with `user` read from a
  file when the setup names one (section 8).

## 5. Managed: reconcile

`temper forge reconcile CONFIG.json FORGE`, for a forge whose setup is
managed or owned.

- **Its inputs:** the forge's `api` and `transport` (configuration.md,
  section 3); the admin's token, from the setup's file when managed, or
  `admin_token` when owned; the bot's login and email; the orgs temper
  works in; the webhooks' listener, when configured.
- **What it converges,** in this order, each read first and written only
  where it differs:
  1. **The admin.** `GET /api/v1/user` with the admin's token answers a
     site admin, or reconcile stops, naming the forge.
  2. **The bot.** The user of the bot's login, made when absent (`POST
     /api/v1/admin/users`, with a random password that is never kept),
     never a site admin, its profile's description carrying the marker
     `temper:bot`. A user of that login without the marker is refused,
     naming the login, so a login written wrong never takes over a
     person's account; one that is a site admin is refused too.
  3. **Its two tokens.** Each of `api_token` and `git_token` is kept when
     its file answers `GET /api/v1/user` as the bot. Otherwise it is made
     again: the bot's password reset to a fresh random one (`PATCH
     /api/v1/admin/users/{login}`), the token of its name (`temper-api`,
     `temper-git`) deleted and made again under basic auth with that
     password (`/api/v1/users/{login}/tokens`), with the scopes of
     section 4, its file written, the password forgotten.
  4. **The bot's id,** written to `user` when it differs.
  5. **Each org,** which must exist when managed, and is made, owned by
     the person, when owned (6.5). In it, a team named the bot's login,
     with write permission on every repository the org has or will have
     (`includes_all_repositories`), the units temper uses (code, issues,
     pull requests and actions), and the bot its member. Adopting any
     repository of the org then finds the bot's write access with no
     admin act at runtime.
  6. **Each org's webhook,** when a webhooks' listener is configured:
     the org's hook whose URL is the listener's path for this forge
     (forge.md, section 5), made when absent, and patched at every
     reconcile with the secret, the events of forge.md, section 5, JSON
     and active, since Forgejo never shows a hook's secret. The secret is
     drawn into `webhook_secret` when that file is missing, and kept
     otherwise. Another hook of the org is left alone.
- **From Forgejo, every time.** Reconcile keeps no record of its own: the
  forge is its record. A second run with nothing changed writes only the
  hooks' patches. A bot deleted, a token revoked, a hook deleted or a
  team a person edited is made again at the next start.
- **It waits for the forge:** its first request is made again after a
  backoff, doubling, within its deadline, until the API answers. So a
  host may start it beside Forgejo with no readiness probe of its own.
- **Bounded:** the orgs at most `setup.orgs`; every listing paged, with
  the forge's page cap; the run within `setup.deadline_ms`, its first
  request made again from `setup.backoff_ms` (configuration.md, section
  7). A failure stops
  it with exit status 1, naming the step, and leaves each file it wrote
  whole and valid.
- **Only at a start.** The admin's token is read by `temper forge
  reconcile` alone, which ends when converged; `temper serve` never reads
  it. With no admin's token, a forge is connected.
- **Repositories outside the orgs,** a person's own among them, are
  adopted as connected's are: the person makes the bot a collaborator,
  and the repository polls unless the person adds its hook.
- **What it cannot set:** Forgejo's settings, which its API does not
  change: the webhooks' `ALLOWED_HOST_LIST`, and whether Actions are
  enabled. A managed forge whose host list refuses temper's listener
  delivers no webhook, and polling keeps up (section 10).

## 6. Owned: prepare and run

### 6.1 Its inputs and its binaries

- **Inputs:** the Forgejo binary's absolute path, and the runner's, or
  no runner; the address Forgejo listens on, a socket address on
  loopback; the person's login and email; the orgs; the runner's labels,
  capacity and `PATH`; the forge's directory.
- **The binaries are checked** by every command that runs them: the
  first line of each one's version equal to temper's pin, Forgejo's and
  the runner's, the versions the conformance checks (forge.md, sections 1
  and 10; `tools/forgejo-conformance`). Another binary refuses the
  command, naming its path and what it printed. temper downloads nothing.
- **One pin,** the conformance's (forge.md, section 10), read by owned
  mode too, so the Forgejo temper runs is the Forgejo whose facts it
  checked.

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
  start (forge.md, 3.5).

### 6.3 The preparation

`temper forge prepare CONFIG.json FORGE`, before Forgejo runs. It takes
Forgejo's and the runner's locks (6.4), checks the binaries, and decides by `state`:

1. **The first preparation:** the layout of section 3; `app.ini`;
   `forgejo migrate`; temper's admin, `temper-admin`, made as a site
   admin with a password Forgejo draws and nobody keeps, and its token
   made by `admin user generate-access-token` into `admin_token`; the
   person's account, a site admin with a password Forgejo draws into
   `person_password`, to change at their first sign-in; the runner's
   secret drawn into `runner/secret` (40 hexadecimal digits, no line
   feed), registered as a global runner by `forgejo-cli actions register
   --secret-file`, and the runner's connection written from it into the
   runner's configuration (`server.connections`), its token in a private
   file the connection names (`token_url: file:`); the work path's
   repositories directory, which `forgejo dump` needs; `state` written
   last, so a preparation cut short is made again from the start.
2. **A later preparation at the same versions:** `app.ini` written again,
   so a change of the setup reaches Forgejo; `admin_token` made again
   when missing, and temper's admin with it; nothing else.
3. **An upgrade:** a recorded Forgejo older than the pin is dumped
   (`forgejo dump`) into `backups/`, named by both versions, then
   migrated, then recorded. A recorded version newer than the pin
   refuses: Forgejo's migrations only go forward.

- **Forgejo's command line only,** on the work path, never its web: under
  the lock no Forgejo runs, so nothing writes its database beside the
  preparation.
- **No secret in an argument.** A password is drawn by Forgejo (the user
  creation's random password) and read from its output; the runner's
  secret goes by file (`--secret-file`). The runner's
  `create-runner-file` takes its secret only as an argument, so the
  preparation never runs it: it writes the runner's connection itself, in
  the runner's configuration, with its token in a file (13.3).
- **Its commands' output,** kept for a failure's message, is masked of
  every secret it holds, as the throwaway Forgejo masks its own.

### 6.4 Running

`temper forge run CONFIG.json FORGE forgejo|runner`:

- **Checks** the binary, and that `state` records a preparation at the
  pinned versions; refuses otherwise, naming what to run.
- **Takes the lock,** an exclusive lock, taken without waiting, on a
  file of the forge's directory, Forgejo's or the runner's, and hands it
  across the exec as the process's standard input, so the process holds
  it for its life. Two Forgejos never share a work path, and no
  preparation runs under one: the preparation takes the same locks, and
  refuses, naming the forge, while one is held.
- **Replaces itself** (`execve`) by `forgejo web` or `forgejo-runner
  daemon`, with the spec's arguments and the spec's environment alone:
  cleared, then `HOME` in the forge's directory, `PATH`, and git's
  configuration isolated, as the throwaway Forgejo's is. The process a
  host supervises is then Forgejo's own, with its signals and its exit
  status.
- **Outside skein's shell, in one module.** skein's shell spawns but
  never execs, and its locks never pass to a child (skein's
  programming-model.md, section 2.1; io.md, section 5). So the lock and
  the exec are std's, `File::try_lock` and `CommandExt::exec`, with no
  `unsafe`, in one module of `temper-forge-setup-shell`, which the
  preparation's lock uses too. It is the one place temper locks a file
  or uses `std::process` outside skein: a boundary test fails the gate
  when either appears in another module. It moves onto skein when skein's
  shell offers both (section 15).
- **The runner's spec:** its configuration file (its registration's file,
  capacity, labels, timeouts, its jobs' directory, no cache), its `PATH`
  from the setup. Its jobs run as the user that runs it (section 15).

### 6.5 Then reconcile, and the person

- **Reconcile runs as managed's** (section 5), with `admin_token`, and
  makes each org that is missing, owned by the person (`POST
  /api/v1/admin/users/{person}/orgs`).
- **The person** signs in on Forgejo's web as a site admin, makes
  repositories in the orgs, with Actions, and adopts them through
  temper's client. temper's admin is temper's; a person who deletes it
  makes the next reconcile stop, naming the admin's token, and a
  preparation with `admin_token` removed makes it again.
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
├── temper forge reconcile CONFIG.json 1      once, per managed or owned forge
├── temper forge reconcile CONFIG.json 2
└── temper serve CONFIG.json                  long: its own tree (protocol.md, section 4)
```

### 7.2 What it does

- **In order:** each step starts once the one before it has started, when
  that one is long, or has exited 0, when it is once.
- **Restarts** a long step that exits, after a backoff doubling from
  `box.backoff_ms` to `box.backoff_max_ms` (configuration.md, section 7),
  first running again the steps that are once just before it: a Forgejo
  is restarted after its preparation, `temper serve` after its
  reconciles, so a token revoked under a running engine, which stops it
  (credentials.md, section 5), is made again before the engine starts.
- **Gives up,** exit 1, naming the step, when a once step fails or a long
  one exits more than `box.restarts` times within `box.window_ms`.
- **Stops** at a first termination signal: the long steps in reverse
  order, each by a signal, `box.grace_ms`, then, for `temper serve`, a
  second signal, which turns its drain into an abort (protocol.md,
  section 4), then a close. A second termination signal closes them all.
  It exits 0 after a stop it was asked for.
- **Its streams:** `temper serve`'s stdout, the sign-in links, is the
  box's stdout; each step's stderr is the box's, each line prefixed by
  the step's name. Forgejo logs to its own files.
- **Its children** are skein-io's (skein's `io.md`, section 6), each in a
  process group of its own; closing one kills its group, so nothing the
  box started outlives its stop.

### 7.3 What it names

- **Steps only:** each step's name, its arguments and whether it is once
  or long. Its policy is a small machine of its own (steps, exits and
  timers in; spawns, signals and closes out), hosted on skein-io.
- **The binary hands it the steps.** `temper box` checks the document
  whole, as `temper serve` does, asks `temper-forge-setup` for its steps,
  and gives them to `temper-box`.
- **Checked:** a boundary test fails the gate when `temper-box` depends
  on a crate named `temper-*` but `temper-forge-setup`, or names an item
  of it but `steps` and `Step`.

### 7.4 Other arrangements of the same commands

Each needs nothing below the commands to change:

- **systemd,** a unit per long step: `prepare` and `reconcile` as
  `ExecStartPre=`, `run` and `temper serve` as `ExecStart=`, the order as
  the units' dependencies;
- **the process tier** (section 13.4), which runs the commands itself;
- **QA's environments** (`draft/qa.md`, 4.1), whose "temper in a box" is
  an owned setup with a scripted LLM, made by its own host from the same
  commands;
- **a container image** whose entry point is `temper box`.

## 8. The configuration

- **One document still,** `temper serve`'s (configuration.md, section 2).
  Every `temper forge` command and `temper box` read it whole, check it
  as `temper sign-in` does, `setup` among it, and act on their part
  (configuration.md, section 8): they read no file but their own part's,
  since a set-up forge's files do not exist before reconcile makes
  them.
- **`application.setup`,** a list, at most one entry per forge:

  ```
  "setup": [
    {"forge": 1, "directory": "forge-1",
     "bot": {"login": "temper", "email": "temper@example.invalid"},
     "orgs": ["acme"],
     "mode": {"managed": {"admin_token": "secrets/forgejo-admin"}}},
    {"forge": 2, "directory": "forge-2",
     "bot": {"login": "temper", "email": "temper@example.invalid"},
     "orgs": ["work"],
     "mode": {"owned": {
       "forgejo": "/opt/forgejo/forgejo",
       "runner": {"binary": "/opt/forgejo/forgejo-runner",
                  "labels": ["temper-host:host"], "capacity": 1,
                  "path": "/usr/local/bin:/usr/bin:/bin"},
       "listen": {"socket": "127.0.0.1:3000"},
       "person": {"login": "ada", "email": "ada@example.invalid"}}}}
  ]
  ```

  | Member | Type | Bounds |
  |---|---|---|
  | `forge` | u16 | the `number` of one entry of `forges`, of kind Forgejo |
  | `directory` | a path | a private directory (section 3), unique among `setup` |
  | `bot` | `{"login", "email"}` | a Forgejo login; an email |
  | `orgs` | list of strings | at least one, at most `setup.orgs`, each a Forgejo org name |
  | `mode` | `{"managed": {"admin_token": file}}` or `{"owned": ...}` | |
  | `owned.forgejo`, `owned.runner.binary` | absolute paths | checked against the pins of 6.1 |
  | `owned.runner` | the runner's members, or null for none | `capacity` at least 1; `labels` at least one |
  | `owned.listen` | an address, jig's format | a socket address on loopback; unique among the deployment's listeners and owned forges |
  | `owned.person` | `{"login", "email"}` | a Forgejo login other than the bot's and `temper-admin` |

- **The forge's own members stay connected's,** written out, never
  derived, and checked against its setup; a mismatch refuses every
  command, naming the path:
  - its `api_token` is `<directory>/api_token` and its `user` `{"file":
    "<directory>/user"}`; its identity's `token` is
    `<directory>/git_token` and its `login` the bot's; its
    `webhook_secret`, when `webhooks` is configured,
    `<directory>/webhook_secret`;
  - when owned, its `api` is `listen`, its `transport` plaintext and its
    `git` `http://<listen>`.
- **`user` as a file:** `forges[].user` takes a u64, or `{"file": path}`,
  a private file holding the id in decimal, read at start. The check
  against `GET /api/v1/user` is the same (credentials.md, section 5).
- **`webhooks`,** `application.webhooks`, `{"listener": address}`, on
  loopback, and each forge's `webhook_secret`, a file (configuration.md,
  section 3; forge.md, section 5).
- **`temper serve` checks `setup`** and acts on none of it: it reads no
  file that only `setup` names.
- **An owned example** joins configuration.md's formats (section 10), for
  a person, a test or QA's host to start from: one owned forge with a
  runner, webhooks, one git identity, one worker and one LLM account.
  Its three loopback addresses (Forgejo's, the clients' listener's and
  the webhooks' listener's) are the ones a host changes to run several
  boxes on one machine.

## 9. Credentials

| Credential | Mode | Made by | Kept | Read by |
|---|---|---|---|---|
| a site admin's token | managed; owned | the person; the preparation, by Forgejo's command line | a private file | `temper forge reconcile` alone |
| the bot's password | managed, owned | reconcile, random, at each token it makes | nowhere | the requests that make that token |
| temper's admin's password | owned | Forgejo, random | nowhere | nothing |
| the person's first password | owned | Forgejo, random, at the first preparation | `person_password` | the person |
| the runner's secret | owned | the preparation | `runner/secret` | Forgejo's command line; the runner's connection, through the token's file it names |
| Forgejo's own secrets | owned | the preparation | `state`, and `app.ini` | Forgejo |
| the webhooks' secret | every mode | the person; reconcile | `webhook_secret`, or the person's file | the webhooks' listener; Forgejo's hooks |

- **Least power, kept.** The engine holds the bot's two tokens and the
  webhooks' secret, as in connected, and never an admin's token; the bot
  is never a site admin; the admin's token is read by a process of its
  own, which ends.
- **No secret in an argument, a URL or a log,** for Forgejo's command
  line as for git (credentials.md, section 4): 6.3.
- **Never across a domain boundary:** the setup's processes are outside
  the engine; the engine's rules (credentials.md, section 1) hold for the
  files it reads.

## 10. Webhooks on a box

The listener, the signature, the events and the hints they become are
forge.md, section 5's. This section says what makes them reach the
engine on a box, and that they cover what its work waits on.

- **Made by reconcile** (section 5, step 6): one hook per org and forge,
  with the events of forge.md, section 5, and the forge's secret. An
  owned Forgejo's `app.ini` allows the listener's address (6.2). On a
  box, nothing is left to the person.
- **What a box's work waits on, and what ends each wait,** with the keep
  reading the forge within `hinted` of the hint, which gathers one
  change's hints into one read (`domain/forge.md`, 5.2):

  | The domain waits on | Done on the forge by | Its delivery | Its hint |
  |---|---|---|---|
  | a landing branch's tip | a merge, a person's push | `push` | the branch moved |
  | a pull request's head, base or state | a push to its head or base; closing, reopening, retargeting or editing it | `push`; `pull_request` | the branch moved; the item |
  | CI on a head | each Actions job ending | the job's end | the commit |
  | a participating object's comments and reviews | a comment, a review, an approval or a rejection | the item's comment or review events | the item |
  | an issue of a live goal | editing, closing, labelling or assigning it | `issues` | the item |
  | drift of a branch temper holds | a push, a deletion, a creation | `push`; `delete`; `create` | the branch moved; the branch |

- **Echoes go up too.** Forgejo delivers temper's own writes; each costs
  the keep one read, and the domain drops what it already knows
  (`domain/forge.md`, 5.2).
- **What Forgejo settles after it delivers,** a pull request's
  mergeability and perhaps a job's commit status (13.3), is read again
  at `hinted`, doubling, within `settle` of the move, and a pull request
  that reads not mergeable within `settle` of the move is never taken as
  conflicted (`domain/forge.md`, 5.2).
- **What sends no delivery,** and so is found by polling alone: a
  commit status that a system other than Actions posts, which a
  box has none of.
- **Polling is unchanged** (forge.md, section 6): the keep's alarms, its
  paces and the slow pass are what they are with no webhook. A delivery
  lost while `temper serve` is down, refused for its size, or never sent
  is found by them; Forgejo does not send a failed delivery again.
- **Checked:** the process tier's hinted story (13.4) runs with the
  keep's paces raised past its budget, so it lands only if every wait
  ends by a delivery.

## 11. The crates

```
temper-forge-forgejo         + the setup's documents: admin users, tokens under basic auth, orgs, teams
│                              and their members, org hooks; both sides, as its others; the pins
temper-forge-setup           step code: the setup's types and checks; reconcile's machine; the
│                              preparation's machine; app.ini and the runner's files; the processes'
│                              specs; the configuration's steps. Names no host
├── temper-forge-forgejo
└── jig-protocol-config, skein-lib, skein-json
temper-forge-setup-shell     ordinary Rust: the hosts of prepare, run and reconcile: commands, files,
│                              the forge's locks, the exec; HTTP on skein-http over skein-io
├── temper-forge-setup
└── skein-io, skein-http, skein-tls, skein-shell
temper-box                   the box's machine (step code) and its host (ordinary Rust) on skein-io's
│                              processes; nothing of setup but its steps (7.3)
├── temper-forge-setup
└── skein-lib, skein-io, skein-shell
temper                       the binary: temper forge prepare|run|reconcile, temper box
```

- **The binary checks `setup` and its limits** for every command,
  `temper serve`'s included: `temper-forge-setup` checks `setup` and the
  `setup.` limits, and `temper-box` the `box.` limits, beside
  temper-service's startup. temper-service depends on neither.
- **Machines, not scripts.** Reconcile and the preparation decide from
  what they read, and say each request or command they need; a host runs
  it and hands back its outcome. Step tests drive them with answers;
  reconcile's world drives it against the fake forge.
- **Not the throwaway's code.** `testing/temper-throwaway-forgejo`, with
  its blocking HTTP and serde, stays a test crate. The production
  machines are written on skein, under the subset; the process tier moves
  to them (13.4).
- **The fake forge serves the setup's documents** too
  (`testing/temper-fake-forge-protocol`), from the server's side of
  `temper-forge-forgejo`, as it serves the engine's.

## 12. Where it serves

- **The first milestone's setup** (`draft/mvp.md`, 5.1): write the
  document from the owned example, `temper sign-in`, `temper box`; make
  a repository with Actions in an org on Forgejo's web, and adopt it.
- **CI for the first milestone** (`draft/mvp.md`, section 7): Actions on
  the owned runner.
- **The process tier** (testing.md, 2.6), on owned mode in place of its
  own provisioning, with CI and webhooks (13.4).
- **QA's environments** (`draft/qa.md`, 4.1), each an owned setup made
  from the same commands (7.4): a box an agent starts, drives through
  temper's client and Forgejo's API as the person, reads the logs of in
  its files, and throws away.

## 13. Testing

### 13.1 Step tests

- **Forgejo's and the runner's files:** `app.ini` and the runner's
  configuration for each setup, as golden text.
- **The preparation's decisions:** a first preparation; one at the same
  versions; an upgrade, dumped before migrated; a newer recorded version
  refused; another binary refused; a missing `admin_token` made again; a
  preparation cut short made again from the start.
- **The specs:** each command's arguments and environment, and no secret
  of the preparation's in any argument.
- **Reconcile's decisions** from answers: each step read, then written
  only where it differs; the marker's refusal; a site admin bot refused;
  a revoked token made again; the hooks patched.
- **The configuration:** each mismatch of section 8 refused, naming its
  path; `steps` for each arrangement of modes.
- **The box's machine:** the order; a restart after its once steps; the
  backoff and the cap; the stop's order, and `temper serve`'s second
  signal.

### 13.2 Reconcile's world

On the fake forge's protocol layer, serving the setup's routes:

- an empty forge converged; a second run writing only the hooks'
  patches;
- a deleted bot, a revoked token, a removed team member, a deleted hook,
  each made again;
- a missing org refused when managed, made when owned;
- a person's account under the bot's login refused;
- no secret in a failure's message or a log line.

### 13.3 Facts to check on v17.0

By `tools/forgejo-conformance`, on the throwaway Forgejo and runner,
before a machine relies on each:

- **The command line,** as known of v17.0 and the runner v13.2, recorded
  as facts: `admin user create --random-password` prints `generated
  random password is '<password>'` on its first line; `admin user
  generate-access-token --raw --scopes` prints the token alone;
  `migrate` runs on a fresh work path; `forgejo dump` needs the work
  path's repositories directory; `forgejo-cli actions register
  --secret-file` takes exactly 40 hexadecimal digits with no line feed,
  and run again with the same secret names the same runner; the runner's
  `create-runner-file` takes its secret only as `--secret`, and is
  deprecated for the configuration's `server.connections`, whose token a
  file may hold (`token_url: file:`).
- **The runner's connection:** how a connection's `uuid` and `token`
  follow from the secret that `forgejo-cli actions register` registered,
  and that the runner started on that configuration alone takes the
  throwaway instance's jobs.
- **The admin routes:** making a user, resetting its password (the
  members `PATCH /admin/users/{login}` requires), its description; making
  a token under basic auth, and deleting one by name; the scopes each
  route of forge.md, section 3.2 needs.
- **Orgs and teams:** making an org for a user, a team with
  `includes_all_repositories` and its units, a member; that a repository
  made later in the org is writable by the team's members.
- **Org hooks:** making one with its events, its secret never shown, and
  its patch; that `ALLOWED_HOST_LIST` refuses loopback by default and
  allows the listener's address when named.
- **The deliveries** of forge.md, section 13: each event of forge.md,
  section 5, its headers and the fields its hint reads; whether a job's
  delivery comes after its commit status is written; whether a pull
  request's mergeability is settled when the push that moved its head or
  base is delivered.

### 13.4 The process tier

- **Owned in place of the test's support** (testing.md, 2.6): the test
  runs `temper forge prepare`, `run forgejo`, `run runner` and
  `reconcile` itself, then `temper serve`, with the binaries
  `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name, and webhooks on.
  Its repository runs Actions, so its landing rule asks for CI. It aims
  at 3 minutes; if CI takes it past, the CI story becomes a test of its
  own.
- **The hinted story:** the same story with the keep's `poll`,
  `poll_max` and `slow` raised past the story's budget (configuration.md,
  section 7), so that it lands within its budget only if every wait ends
  by a delivery and the reads that follow it; and with no conflict
  resolved, since its changes never conflict.
- **`temper box`,** a test of its own on a scratch directory: the sign-in
  link on its stdout; Forgejo killed and restarted, with the engine still
  serving; a stop leaving no process it started.

## 14. Order of building

After the `protocol` plan, beside the first milestone's (`draft/mvp.md`,
section 6), which it serves:

1. **Facts** of 13.3, by the conformance tool.
2. **HMAC-SHA256 and a comparison in constant time** in skein's lib
   (skein's `lib.md`, 6.2), with its SHA-256, unless another plan has
   built them first.
3. **Webhooks** (forge.md, section 5): the listener, the signature, the
   hints, `application.webhooks` and each forge's `webhook_secret`; the
   fake forge's protocol layer sending deliveries; the throwaway
   Forgejo allowing the listener, with a hook its support makes.
4. **Owned's preparation,** and `temper forge prepare` and `run`; the
   process tier's Forgejo started by them.
5. **Reconcile,** and `temper forge reconcile`; `user` as a file; the org
   hooks; the process tier's provisioning by reconcile, its runner by
   `run`, CI on, and the hinted story.
6. **`temper box`,** and its test.
7. **The owned example,** and the first milestone's setup document on
   it.

## 15. Later

Each waits for what it names; none is built by the order of section 14.

- **GitHub.** What the modes share: temper makes itself an integration,
  an identity, its credentials, the delivery of its events and the
  repositories it may reach. Connected: a fine-grained token, or an App
  the person makes and installs. Managed: the App manifest flow, which
  reconcile drafts and the person creates on GitHub; installing the App
  is its scope, in place of teams. An installation's token expires within
  the hour, so it is a credential the forge component renews (forge.md,
  section 11). Owned: none. Waits for GitHub's build.
- **A lock and an exec in skein's shell,** onto which 6.4's module
  moves. Waits for skein's `Lock` (io.md, section 5), and for a second
  service that execs.
- **The binaries' digests,** an optional SHA-256 per binary beside the
  version's check. Waits for a deployment that installs binaries it did
  not build or download itself.
- **Webhooks from another host,** over TLS (forge.md, section 5). Waits
  for skein's TLS server side.
- **CI's isolation.** An owned runner's jobs in host mode run as the
  box's user, who can read every file of temper's, its tokens among them,
  as credentials.md, section 8 accepts of agents in standalone. Until
  then, an owned forge runs the person's and temper's agents' code only,
  never a stranger's pull request.
- **Provisioning at adoption** (`domain/connectors.md`, section 11): a
  repository in a person's own namespace, made writable and hooked
  without the person.
- **The forge as people's sign-in provider** (credentials.md, section 6),
  which would link the person's account on an owned Forgejo to their
  owner in jig.

## 16. Open questions

- **A command that writes the document** (`temper box init`). Default:
  the owned example only.
- **Overrides of `app.ini`.** Default: none; the settings are temper's.
- **A token for the person's scripts and agents.** Default: none; the
  person makes one on Forgejo's web.
- **Backups beyond an upgrade's.** Default: the person's, of the forge's
  directory and the store, with the box stopped.
- **Two deployments on one managed forge.** Default: each its own bot
  login, and so its own team and hooks; the marker does not tell
  deployments apart.
- **A hook left at an old listener's address,** after the listener
  moves. Default: left alone; the person deletes it.
- **A managed forge whose host list refuses the listener.** Default:
  nothing tells the person but the hints that never arrive
  (`domain/forge.md`, section 4).
