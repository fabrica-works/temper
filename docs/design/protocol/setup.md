# The forge's setup

Provisional, 2026-10-16, revised 2026-10-18. How a forge comes to be
ready for temper, in three modes: connected, where a person runs and sets
up Forgejo and gives temper only what it strictly needs; managed, where a
person runs it and gives temper a site admin's token, with which temper
sets itself up; and owned, where temper runs Forgejo and its runner, and
sets itself up on them. The engine is the same in every mode: each mode
makes the files the one below reads, and the engine reads connected's.
Owned mode is commands any host can arrange; `temper box` is one
arrangement, and wiring only. A box is what temper's first milestone is
set up on and checked by CI on (`../README.md`, section 2), and what QA's
agents drive (`../draft/qa.md`, 4.1). The members are configuration.md's
(section 3), the credentials credentials.md's (section 2), the webhooks
forge.md's (section 5), and the commands and crates README.md's
(sections 4 and 5). What comes later is section 15; what is open,
section 16.

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
- **The core is step code; the hosts are thin.** Reconcile is a machine
  of skein's `skein-forge-setup` (`forge-setup.md`, section 4); the
  preparation, Forgejo's settings and the processes' specs are machines
  and pure functions of `temper-forge-setup`. Running a command, a
  request or a file write is a host's. Three commands host them: `temper
  forge prepare`, `temper forge run` and `temper forge reconcile`
  (sections 5, 6 and 11).
- **`temper box` is wiring only.** It runs a configuration's steps in
  order (prepare, Forgejo, its runner, reconcile, `temper serve`),
  restarts what exits, and stops them in reverse order. It names no
  Forgejo request and nothing of setup but its steps. systemd, QA's
  environments or a test arrange the same commands another way
  (section 7).
- **Reconcile converges at every start:** the bot user, made when
  missing and otherwise checked, never edited; its two tokens, a team
  with write on every repository of each configured org, the bot in it,
  and each org's webhook, each made when missing, repaired when changed
  and left alone otherwise. The admin's token is read then, by `temper
  forge reconcile` alone (section 5).
- **Owned is a stock Forgejo.** temper writes its `app.ini`, runs its
  migrations, makes its own admin and the person's account with
  Forgejo's command line, registers the runner offline, dumps before an
  upgrade's migration, and runs only the versions it pins, the
  conformance's (section 6). Its directory, once temper stops running
  it, is a Forgejo a person can run themselves, as managed.
- **A box is reactive.** Every change on the forge that a box's work
  waits on reaches the engine as a signed webhook delivery, and becomes a
  hint, read within the keep's `hinted`: pushes, pull requests, issues,
  comments, reviews and each CI job's end. Reconcile makes the hooks; an
  owned Forgejo is allowed to deliver to the engine's listener. Polling
  is unchanged, the fallback for a delivery lost or refused (section 10).
- **Loopback.** An owned Forgejo listens on loopback, as temper's
  clients' and webhooks' listeners do; a person on another machine
  reaches them through a tunnel. A forge on another host polls: its
  webhooks need skein's TLS server side (section 15).
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

The three modes and their layering (each mode makes the inputs of the
one below, and the service always runs connected) are held by skein's
`forge-setup.md`, section 3. temper's layers, and the files that pass
between them:

```
owned      temper forge prepare     ──► the forge's work path, the runner's, admin_token, person_password
           temper forge run         ──► forgejo web, forgejo-runner daemon (each exec'd)
managed    temper forge reconcile   ──► api_token, git_token, user, webhook_secret
connected  temper serve             ◄── reads them, through the forges and identities that name them
```

- **One directory per set-up forge,** named by its setup (section 8):
  private, `0o700`, the user's own and not reached through a link, as a
  worker's root is (configuration.md, section 4). The first command that
  writes in it makes it so when it is absent: `temper forge prepare`
  when owned, `temper forge reconcile` when managed. A directory that is
  not so refuses the command, naming it. Its files:

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
  webhooks, their secret. The scopes (skein's `forge-setup.md`, section
  8): `read:user`, `write:issue` and `write:repository` for the API token;
  `write:repository` and `read:user` for the git token, the second so
  that reconcile can ask whose it is (skein's `forge-setup.md`, section
  4.3, step 3).
- **What the person does:** a Forgejo user for temper, not a site admin;
  its two tokens; its write access to each repository temper will adopt,
  as a collaborator or through a team; optionally, each repository's or
  org's webhook to temper's listener, with the secret and the events of
  skein's `forge.md`, section 7, and Forgejo's `[webhook]
  ALLOWED_HOST_LIST` letting it reach the listener; CI, as each
  repository has it.
- **Nothing more is required** (`../domain/forge.md`, section 4): polling
  stands in for webhooks, and a project's checks for CI.
- **The engine as configuration.md has it,** with `user` read from a
  file when the setup names one (section 8).

## 5. Managed: reconcile

`temper forge reconcile CONFIG.json FORGE`, for a forge whose setup is
managed or owned.

- **The machine is skein's.** Its six steps, in this order (1 the
  admin; 2 the bot, marked `<word>:bot`; 3 the bot's two tokens; 4 the
  bot's id; 5 each org's team, with the bot in it; 6 each org's
  webhook), each read first and written only where the forge differs;
  its wait for the forge; its bounds; and what it refuses are held by
  skein's `forge-setup.md`, section 4. It does no io and reads no
  configuration: it is given values, says each request, and its host
  runs it. temper's setup shell is that host (section 11).
- **Its given values, from temper's configuration:**

  | Reconcile's value | temper's |
  |---|---|
  | the forge's number | the forge's number (configuration.md, section 3) |
  | the bot's login and email | the setup's `bot.login` and `bot.email` (section 8) |
  | the tokens' names | `temper-api` for the API's, `temper-git` for git's |
  | the orgs | the setup's `orgs` |
  | the orgs' owner | `mode.owned.person.login` (6.5) when owned; none when managed |
  | whether an absent org is made | yes when owned; no when managed |
  | the name a failure gives the admin | `temper-admin` |
  | the listener's address | `application.webhooks.listener`'s address, when configured; none otherwise. The hook's URL is derived from it (skein's `forge-setup.md`, section 4.1) |
  | the word | `temper`, so the bot's mark is `temper:bot` |
  | its limits | `setup.orgs`, `setup.deadline_ms`, `setup.backoff_ms` and `setup.backoff_max_ms` (configuration.md, section 7) |

  The API's address is the host's: the forge's `api` (configuration.md,
  section 3).
- **What the host answers, as needs:**

  | Reconcile asks for | The setup shell answers |
  |---|---|
  | the admin's token | the file the setup's `mode.managed.admin_token` names when managed; `admin_token` in the forge's directory when owned (section 3) |
  | the API and git tokens, the bot's id | the content of `api_token`, `git_token` and `user`, or none when the file is missing |
  | the webhooks' secret | the content of `webhook_secret`, or none when the file is missing, and reconcile then draws one |

- **Plaintext to loopback alone:** the forge's `transport` is
  plaintext, and a setup whose forge's transport is TLS is refused,
  naming it (section 15).
- **Its files** (section 3). The host reads the admin's token and, when
  they exist, `api_token`, `git_token`, `user` and `webhook_secret`, as
  configuration.md, section 6 reads a secret's file, each within its
  bound; writes the four whole when reconcile asks; and draws the
  random bytes reconcile asks for, which become `webhook_secret` when
  that file is missing. An existing `webhook_secret` is kept.
- **The bot's email** is the setup's `bot.email` when reconcile makes
  the bot, and stays as it is after: a change to `bot.email` reaches
  only a bot made after it, and an existing bot's email is the
  person's to change, on Forgejo. temper reads no mail of the bot's,
  and its commits name the identity's `author` (configuration.md,
  section 3), not this email.
- **The scopes** of the tokens are section 4's.
- **A failure** stops the command with exit status 1, naming the step,
  and leaves each file it wrote whole and valid.
- **Only at a start.** `temper serve` never reads the admin's token, and
  reconcile ends when converged. With no admin's token, a forge is
  connected.
- **Repositories outside the orgs,** a person's own among them, are
  adopted as connected's are: the person makes the bot a collaborator,
  and the repository polls unless the person adds its hook.
- **What it cannot set:** Forgejo's settings, which its API does not
  change (skein's `forge-setup.md`, section 4): the webhooks'
  `ALLOWED_HOST_LIST`, and whether Actions are enabled. A managed forge
  whose host list refuses temper's listener delivers no webhook, and
  polling keeps up (section 10).

## 6. Owned: prepare and run

### 6.1 Its inputs and its binaries

- **Inputs:** the Forgejo binary's absolute path, and the runner's, or
  no runner; the address Forgejo listens on, a socket address on
  loopback; the person's login and email; the orgs; the runner's labels,
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
   made by `admin user generate-access-token` into `admin_token`, with
   the scopes `write:admin`, `write:organization` and `write:user` (the
   scopes the conformance records reconcile's routes under; skein's
   `forge-setup.md`, section 8, says which route needs which), under a
   name that ends in a random suffix, since Forgejo refuses a second
   token of one name; the
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
   under a fresh suffix, for temper's admin, which Forgejo's command line
   cannot make twice: an admin a person deleted fails the preparation,
   naming it; nothing else. It takes Forgejo's lock alone, so a box
   restarts a Forgejo, after its preparation, while the runner runs.
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
  do and as reconcile's HTTP runs: each spawned with its arguments and
  the spec's environment, its output read through pipes within a bound,
  its exit status awaited.
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

### 6.5 Then reconcile, and the person

- **Reconcile runs as managed's** (section 5), with `admin_token`, and
  makes each org that is missing, owned by the person (`POST
  /api/v1/admin/users/{person}/orgs`).
- **The person** signs in on Forgejo's web as a site admin with
  `person_password`, and changes it, since Forgejo's API refuses them
  until they do (13.3); makes repositories in the orgs, with Actions,
  and adopts them through temper's client. temper's admin is temper's; a
  person who deletes it makes the next reconcile and the next
  preparation stop, naming it. A preparation with `admin_token` removed
  makes a new token for it.
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
└── temper serve CONFIG.json                  long: its own tree (README.md, section 4)
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

- **systemd,** a unit per long step: `prepare` and `reconcile` as
  `ExecStartPre=`, `run` and `temper serve` as `ExecStart=`, the order as
  the units' dependencies;
- **the process tier** (section 13.4), which runs the commands itself;
- **QA's environments** (`../draft/qa.md`, 4.1), whose "temper in a box" is
  an owned setup with a scripted LLM, made by its own host from the same
  commands;
- **a container image** whose entry point is `temper box`.

## 8. The configuration

- **One document,** `temper serve`'s (configuration.md, section 2).
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
  | `bot` | `{"login", "email"}` | a Forgejo login; an email, the bot's when reconcile makes it (section 5) |
  | `orgs` | list of strings | at least one, at most `setup.orgs`, each a Forgejo org name |
  | `mode` | `{"managed": {"admin_token": file}}` or `{"owned": ...}` | |
  | `owned.forgejo`, `owned.runner.binary` | absolute paths | checked against the pins of 6.1 |
  | `owned.runner` | the runner's members, or null for none | `capacity` at least 1; `labels` at least one, at most 8 |
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
  section 3; skein's `forge.md`, section 7).
- **`temper serve` checks `setup`** and acts on none of it: it reads no
  file that only `setup` names.
- **An owned example** ships for a person, a test or QA's host to start
  from (8.1).

### 8.1 The owned example

`docs/examples/owned.json` is a whole document, `engine`, `workers` and
`application`, for one owned forge with a runner, webhooks, one git
identity, one worker and one LLM account handed in (credentials.md,
section 3). The files it names that hold no secret are beside it, in
`docs/examples/owned/`: the agent's configuration and the charters,
short texts a person edits. The files that hold a secret, the worker's
secret and its digest and the LLM account's file, are never shipped:
`docs/guide/box.md` says how each is made. A person, a test or QA's host
starts from a copy, and `temper box CONFIG.json` runs it (section 7).

- **Its forge,** forge 1, is owned, in `forge-1`, with a runner whose
  label is `temper-host:host`, the bot `temper`, the person `ada` and the
  org `acme`. Its members are its setup's, written out (above): `api`
  and `git` from `listen`, plaintext; `api_token`, `user`,
  `webhook_secret` and the identity's `token` in the forge's directory;
  the identity's `login` the bot's.
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
- **Its agent's configuration** (configuration.md, section 5),
  `owned/agent.json`, takes the process tier's profile, the sizes its
  live tier runs a real model with: smith's `standard`, with `declared`
  `memory` 2,147,483,648, `tool_payload` 600, `read_window` 256,
  `conversations` 1 and `calls_per_response` 1, and `policy`
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
  and whatever its forge's files hold: each start passes with each
  string its service's worst case counts, and a person's copy changes,
  at its bound (configuration.md, section 7, one worst case per
  service).
  - The engine's counts the forge's `api_token` and its `git` base, a
    byte for each of theirs: the token at `small`'s
    `forge_component.token_bytes`, 512, past which the start is refused,
    and the base at an owned forge's longest, `http://` with a loopback
    address and a port, `http://127.255.255.254:65535`, 28 bytes.
  - The worker's counts its agent's `program` and the configuration's
    path among its `arguments`, two bytes for each of theirs with the
    example's one slot: each at Linux's `PATH_MAX`, 4,096 bytes.
  - Nothing else a copy changes moves either: its directory but for that
    path, its other addresses and secrets, its logins, emails, binaries
    and charters, the account handed in.

  A test starts the example with those strings at their bounds, so that
  the example starts from a person's directory, with Forgejo's tokens,
  on the ports a host picks.
- **Three loopback addresses,** Forgejo's (`listen`, `127.0.0.1:3000`),
  the clients' listener's (`127.0.0.1:7100`) and the webhooks'
  listener's (`127.0.0.1:7101`), are what a host changes to run several
  boxes on one machine, with each box's directories.

## 9. Credentials

| Credential | Mode | Made by | Kept | Read by |
|---|---|---|---|---|
| a site admin's token | managed; owned | the person; the preparation, by Forgejo's command line | a private file | `temper forge reconcile` alone |
| the bot's password | managed, owned | reconcile, random, when it makes the bot | nowhere | nothing |
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

The listener is skein's (`forge-connection.md`, section 6), and so are
the signature, the events and the hints they become (`forge.md`, section
7). This section says what makes them reach the engine on a box, and
that they cover what its work waits on.

- **Made by reconcile** (section 5; skein's `forge-setup.md`, section
  4.3, step 6): one hook per org and forge, with the events of skein's
  `forge.md`, section 7, and the forge's secret. An
  owned Forgejo's `app.ini` allows the listener's address (6.2). On a
  box, nothing is left to the person.
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
  in the worlds (`../testing.md`, 2.2 and 6.3).

## 11. The crates

```
skein-forge                  + the setup's documents: admin users and their tokens, orgs, teams
│                              and their members, org hooks; both sides, as its others; the pins
│                              (skein's forge.md, sections 4 and 5)
skein-forge-setup            reconcile's machine, its types and its checks; the access it ends with.
│                              Names no host (skein's forge-setup.md, section 6)
├── skein-forge
└── skein-lib, skein-json
temper-forge-setup           step code: the setup's types and checks, read from the configuration; the
│                              preparation's machine; app.ini and the runner's files; the processes'
│                              specs; the configuration's steps. Names no host
├── skein-forge
└── jig-protocol-config, skein-lib, skein-json
temper-forge-setup-shell     ordinary Rust: the hosts of prepare, run and reconcile: commands, files,
│                              the forge's locks, the exec; HTTP on skein-http over skein-io. Its
│                              reconcile maps the setup onto the machine's values (section 5)
├── temper-forge-setup, skein-forge-setup, skein-forge
└── skein-io, skein-http, skein-lib, skein-json, skein-shell
temper-box                   the box's machine (step code) and its host (ordinary Rust) on skein-io's
│                              processes; nothing of setup but its steps (7.3); one crate, under the
│                              subset's lints, its host written within the subset
├── temper-forge-setup
└── skein-lib, skein-io, skein-shell
temper                       the binary: temper forge prepare|run|reconcile, temper box
```

- **The binary checks `setup` and its limits** for every command,
  `temper serve`'s included: `temper-forge-setup` checks `setup` and the
  `setup.` limits, and `temper-box` the `box.` limits, beside
  temper-service's startup. temper-service depends on neither.
- **Machines, not scripts.** Reconcile (skein's) and the preparation
  decide from what they read, and say each request or command they need;
  a host runs it and hands back its outcome. Step tests drive them with
  answers; reconcile's world, skein's, drives it against the fake forge
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
- **The process tier** (`../testing.md`, 2.6), on owned mode in place of
  its own provisioning, with CI and webhooks (13.4).
- **QA's environments** (`../draft/qa.md`, 4.1), each an owned setup made
  from the same commands (7.4): a box an agent starts, drives through
  temper's client and Forgejo's API as the person, reads the logs in its
  files, and throws away.

## 13. Testing

The tiers, their budgets, the fakes and the layout are `../testing.md`'s;
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
- **Reconcile's decisions** from answers are skein's (skein's
  `forge-setup.md`, section 7). What is temper's is the mapping of
  section 5: each member of a managed setup and of an owned one becomes
  its value, the word `temper` and the names `temper-api` and
  `temper-git` among them.
- **The configuration:** each mismatch of section 8 refused, naming its
  path; `steps` for each arrangement of modes.
- **The box's machine:** the order; a restart after its once steps; the
  backoff and the cap; the stop's order, and `temper serve`'s second
  signal.

### 13.2 Reconcile's world

The world is skein's (`forge-setup.md`, section 7): reconcile on the
fake forge's protocol layer, serving the setup's routes. What is
temper's is its host of reconcile, checked by the shell's own tests and
by the process tier (13.4):

- the first request made again, with the backoff doubling to its cap,
  until the API answers, and the run ended at `setup.deadline_ms` when
  nothing answers or the forge goes silent after an answer;
- a later request that fails not made again;
- the admin's token sent as a token header;
- an admin's token readable by others refused, naming it;
- each file read within its own bound;
- an io short of the HTTP intake or output refused before any request;
- a refused request, and a deleted admin, ended naming their step, with
  no secret in the message.

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

The tier, its budget and its referee are `../testing.md`'s (2.6). What
it checks of this document:

- **Owned in place of the test's support:** the test runs `temper forge
  prepare`, `run forgejo`, `run runner` and `reconcile` itself, then
  `temper serve`, with the binaries `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name, and webhooks on. Its repository runs
  Actions, so its landing rule asks for CI.
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

- **GitHub.** The setup's plan is skein's (`forge-setup.md`, section 5):
  what the modes share, connected's fine-grained token or App, and
  managed's App manifest flow, which the plan drafts and the person
  creates on GitHub, with installing the App as its scope in place of
  teams. An installation's token expires within the hour, so it is a
  credential the forge component renews (skein's `forge-connection.md`,
  section 5); GitHub's forge component is open (`forge.md`, section 14).
  Owned: none.
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
  box's user, who can read every file of temper's, its tokens among them,
  as credentials.md, section 8 accepts of agents in standalone. Until
  they are isolated, an owned forge runs the person's and temper's
  agents' code only, never a stranger's pull request.
- **Provisioning at adoption** (jig's `domain/connectors.md`, 12): a
  repository in a person's own namespace, made writable and hooked
  without the person.
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
  the agent it runs (`../tools.md`, section 10); never the preparation's
  `admin_token`, which reconcile alone reads (section 9).
- **Backups beyond an upgrade's.** Default: the person's, of the forge's
  directory and the store, with the box stopped.
- **Two deployments on one managed forge.** Default: each its own bot
  login, and so its own team and hooks; the marker does not tell
  deployments apart.
- **A hook left at an old listener's address,** after the listener
  moves, is skein's question (skein's `forge-setup.md`, section 9).
  Default: left alone; the person deletes it.
- **A managed forge whose host list refuses the listener.** Default:
  nothing tells the person but the hints that never arrive
  (`../domain/forge.md`, section 4).
- **The owned example's agent, sized for a person's work.** Default: the
  process tier's profile (8.1), sized for the live tier's stories.
- **The owned example's `memory_bytes`** are `small`'s worst cases with
  its strings at their bounds (8.1): 78,669,036,604,974 bytes for the
  engine and 1,989,738,112,203 for the worker, more than any machine
  has, so the start's check bounds nothing a person's machine holds.
  Default: as the example states them; which terms of the worst cases
  make them so large is open (configuration.md, section 11).
