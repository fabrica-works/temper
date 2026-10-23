# The browser in the binary

Draft, 2026-10-23. temper's binary carries the browser's bundle, so that
`temper box` and `temper serve` serve the browser with nothing beside
the binary, and `cargo build` in a checkout makes that binary. It is a
person's convenience, the user's first: temper's gate, its build
infrastructure and its agents keep building the way that serves their
work best, without it unless it helps them (section 4). It answers
`client/browser.md`'s open question "The bundle in the binary" (section
5), whose default this draft replaces. A spike, a throwaway crate in a
worktree, never landed, built the dev bundle inside `cargo build` as
section 2 says; section 6 holds its numbers. Section 8 holds what is
still open, each with a default.

## 1. In one page

- **One file.** The `temper` binary holds the bundle: the shell's
  module, its glue and the stylesheets and files skein's bundler puts
  beside them (`client/browser.md`, 2.2 and 2.5). Running it is the
  whole install: no `share/temper/bundle`, no `scripts/install.sh`
  step, no `scripts/bundle.sh`.
- **A plain `cargo build` makes it.** `cargo build`, then
  `./target/debug/temper box`, serves the browser. So does `cargo build
  --release`, and `cargo install --git` of temper's repository.
- **For people; the machine's builds choose for themselves.** The embed
  is what a build does when nothing says otherwise. temper's gate builds
  without it, as fast as today, and so does every agent whose work needs
  no browser in the binary: `TEMPER_BUNDLE=none`, set once in the gate's
  script and in the agents' environment, never typed per command
  (section 4). A build without it compiles nothing of it, and the suites
  pass either way (section 5).
- **A crate of its own embeds it,** `temper-client-bundle`, which
  `temper` depends on. Its build script runs a nested cargo, in a target
  directory of its own, that builds `temper-client-shell-web` for wasm32
  and skein's bundler, runs the bundler on the module, and embeds the
  bundle's files (section 2). No artifact dependency, no `xtask`,
  nothing unstable: a build script running cargo for another target is
  the pattern substrate's `wasm-builder` and risc0's `risc0-build` ship.
- **Rebuilt only when what it is made from changes:** the module's
  sources as the nested build's own dep-info names them, the
  manifests, `Cargo.lock`, and the shell's stylesheets and files
  (2.2). An edit elsewhere never runs it.
- **Served as a directory's is.** At start, `temper serve` makes the
  embedded files into the same `Bundle` a directory's read makes,
  through the same checks and within the same limits (section 3).
  `temper-service` does not change: it adopts a bundle, wherever the
  binary got it.
- **`clients.bundle`'s `null` is the binary's own,** and a directory
  still overrides it: the browser tier's bundle, or one built apart
  (section 3). `temper init` no longer looks for a bundle.
- **The embed needs the wasm32 target.** A build with it and without the
  target fails, naming the two ways out: add the target, or build
  without the browser (2.4).
- **What a person's build pays** (section 6, measured): once, the first
  run, about a minute and a half; then, after an edit to the client's
  crates, about 6 s more on a build of `temper`; nothing after any other
  edit, and nothing on a build that does not include `temper`.

## 2. The build

### 2.1 The crate

`crates/temper-client-bundle`: ordinary Rust, as skein's bundler is
(skein's `ui.md`, section 2), with its own `[lints]` table, as tools
have.

- **Its build script,** unless `TEMPER_BUNDLE` is `none` (2.4), takes
  the steps `scripts/bundle.sh` takes, with the cargo running it
  (`CARGO`), from the workspace's root, into the target directory of
  2.2:
  1. `build --locked --target wasm32-unknown-unknown -p
     temper-client-shell-web`, with the profile of 2.3;
  2. `build --locked --release -p skein-ui-bundle`: the bundler,
     optimized (2.3);
  3. runs the bundler with `scripts/bundle.sh`'s inputs (the name, the
     prefix `/bundle/`, `temper.css`, `task-pages.css`, the shell's
     `files/` and the noscript words), into its `OUT_DIR`;
  4. writes, into `OUT_DIR`, the list of the bundle's files, each its
     path and its bytes by `include_bytes!`, which the crate's library
     includes.
- **Its library** is that list and the bundle's digest: data, no code
  but the include. The binary's crate reads it (section 3). With
  `TEMPER_BUNDLE=none`, the list is empty and the binary has no bundle.
- **No build dependencies.** The bundler is built by the nested cargo,
  as `scripts/bundle.sh` builds it, never compiled for the script: a
  build without the embed compiles a build script of `std` alone and
  nothing of the bundler, and nothing of the bundler is ever linked
  into `temper`, as skein's `ui.md` has it ("never linked into a service
  or a module"). One `Cargo.lock` still locks the bundler's
  `wasm-bindgen-cli-support` with the module's macros (skein's
  `ui-web.md`, 4.1).
- **Not in `temper`'s own crate:** a build script that runs makes its
  package build again, and the root crate is the slowest to build.
  The include in a small crate keeps the rerun's own cost small; what
  it still costs `temper` is section 6's.
- **Not a package of its own for `main.rs`,** which would rebuild only
  `main.rs` when the bundle changes: `temper`'s crate already builds
  again after a client edit, since the terminal shell depends on the
  same crates, so the split would save little of section 6's cost, and
  would move `temper`'s integration tests, which run the binary by
  `CARGO_BIN_EXE_temper`, with it.

### 2.2 The nested build's directory and its inputs

- **Its own target directory,** `<target>/bundle/`, `<target>` the outer
  build's, found from `OUT_DIR`. The outer cargo holds its own build
  directory's lock while build scripts run, so a nested build in the
  same directory would wait on it forever. One shared directory, not
  one under each `OUT_DIR`, so that a check, a clippy run, a build and a
  test build of `temper` reuse one module and one bundler, and the build
  after a change is incremental. A build without the embed never makes
  it.
- **None of the outer build's flags or wrappers:** the nested cargo
  runs without `CARGO_ENCODED_RUSTFLAGS` and `RUSTFLAGS` (the host's
  flags, not wasm32's: on the development machine, mold's link
  arguments), `RUSTC_WORKSPACE_WRAPPER` and `CLIPPY_ARGS` (which
  `cargo clippy` sets, and which would make a module of another
  fingerprint, built again at each switch between clippy and build) and
  `CARGO_TARGET_DIR`, set anew. It keeps the jobserver (`CARGO_MAKEFLAGS`),
  so its jobs count within the outer build's, and `heavy`'s limits hold.
- **A check keeps its own run of the script:** under `cargo clippy`,
  cargo builds a workspace member's build script apart, so the first
  clippy run and the first build after a change each bundle once. The
  module is built once, in the shared directory; the second finds it
  built.
- **What reruns the script,** each by `cargo:rerun-if-changed`:
  - each source file the nested build's dep-info names (the `.d` cargo
    writes beside the module): every path package the module is built
    from, temper's client crates and, when a local jig or skein is
    patched in (`docs/development/local-jig.md`), theirs;
  - each of those packages' `Cargo.toml`, the workspace's `Cargo.toml`
    and `Cargo.lock`, which pins the git dependencies;
  - the shell's stylesheets and its `files/` directory;
  - and `TEMPER_BUNDLE`, by `cargo:rerun-if-env-changed`.
  Its first run has no dep-info yet, and runs regardless.

### 2.3 Profiles

- **A dev build embeds a dev module,** as `scripts/bundle.sh` makes
  today: a bundle of 9.0 MB, 8.8 MB of it the module.
- **A release build embeds a module built with `web`,** the profile
  skein's `ui-web.md`, 4.1 names for an application's release module
  (`opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`, `panic =
  "abort"`, `overflow-checks = true`, `strip = true`), which temper's
  workspace defines as skein's does. The script maps cargo's `PROFILE`,
  `debug` or `release`, to these two.
- **The bundler is always built optimized,** whatever the outer
  profile, as skein's `ui-web.md`, 4.1 asks of a tool: on today's dev
  module it bundles in 1.0 s, against 5.3 s (12 s of CPU) unoptimized.

### 2.4 `TEMPER_BUNDLE`, and the wasm32 target

- **`TEMPER_BUNDLE`:** unset or `embed`, the embed; `none`, no nested
  build and no bundle; any other value fails the build, naming the two.
  `embed` is there for a build whose environment says `none` and that
  wants the browser this once.
- **Who builds with `none`:** the machine's builds, the gate's and the
  agents' (section 4); a machine without the target; an editor's own
  builds (rust-analyzer runs build scripts at its start). Such a binary
  serves the page for no bundle, as today's binary does with no bundle
  configured (section 3).
- **The embed without the target:** the nested build fails, and the
  script fails the build with one message: `temper-client-bundle: the
  browser's bundle needs the wasm32-unknown-unknown target: add it
  (rustup target add wasm32-unknown-unknown), or build without the
  browser (TEMPER_BUNDLE=none)`. Never a binary without the browser in
  silence (8.1).
- **Offline:** the nested build may download crates only wasm32 uses,
  as any first build of the target does; an offline build runs `cargo
  fetch` first, which fetches for every target.

## 3. Serving it

- **`application.clients.bundle`** (`protocol/configuration.md`, section
  3): `null` serves the binary's bundle, or, from a binary built
  without one, the page for no bundle; a directory serves that
  directory's bundle, read at start as today, and the binary's is
  unused. The row's words change; its type does not, and every document
  written so far stays valid. One that names `share/temper/bundle` keeps
  serving that directory, older than the binary once it is rebuilt:
  set it to `null`.
- **The embedded bundle at start:** its files, already in the binary,
  are made into `temper-service`'s `Bundle` (its page, its digest, its
  files by path with their types from the closed list, its `ETag`) by
  the same code as a directory's read, given files in memory in place
  of a directory, so the same checks hold: the closed list of types,
  each path's bound, and `clients.bundle_files` and
  `clients.bundle_bytes`, a start past them refused naming the limit,
  as for a directory. The copy is held as a read one is, within what
  the profile's worst case already counts for the bundle
  (`clients.bundle_bytes`), so the memory account does not change.
  The dev bundle is 9.0 MB, against the default's 16 MiB.
- **One version:** a page from the binary's own bundle always speaks
  its engine's records, so a client out of date (`client/README.md`,
  5.1) comes only from a binary replaced under an open tab, or from a
  directory configured.
- **`temper init`** (`protocol/setup.md`, 8.2) no longer looks for
  `<the binary's directory>/../share/temper/bundle` and always writes
  `clients.bundle` as `null`. Its line `init: no bundle at <path>: the
  browser is not served` goes; from a binary built without a bundle it
  says `init: this temper has no browser (built with
  TEMPER_BUNDLE=none): the browser is not served`.
- **The page for no bundle** (`client/browser.md`, 2.2, step 2) stays,
  for a binary built without one. Its words change: "This temper was
  built without its browser client.", "Open the link in a terminal with
  temper open DIRECTORY LINK." and "To serve the browser, build temper
  with the wasm32-unknown-unknown target, or name a bundle's directory
  in application.clients.bundle, and restart temper."

## 4. The machine's builds, and what else changes

The embed is for a person running temper from a checkout or an
install. The machine's builds take it where it helps their work, and
leave it where it costs them.

- **The gate** (`scripts/check.sh`) exports `TEMPER_BUNDLE=none`: its
  clippy runs and its suites build as today, its time does not change,
  and its browser tier names its own bundle, as now (`client/browser.md`,
  4.3).
- **Agents** build with `TEMPER_BUNDLE=none` set once in their
  environment, never in each command: where they run in Claude Code,
  its settings' `env`. temper's `docs/development/workflow.md` says so in
  a line. An agent without the setting pays a person's costs (section 6)
  and fails nothing on a machine with the target. An agent whose work
  wants the browser in the binary, a QA pass in a browser on `temper
  box` or a binary built for the person, builds with
  `TEMPER_BUNDLE=embed`.
- **`scripts/bundle.sh`** stays, for a bundle built apart: the browser
  tier's, which turns on skein-ui-web's `loopback-navigation`
  (`client/browser.md`, 4.3) and so is never the binary's, and a
  developer's. It may share the embed's steps; it never needs the
  embed's directory.
- **`scripts/install.sh`** installs the binary alone, built with
  `--release --locked`, and prints its path.
- **The browser-box workaround goes:** a bundle copied to
  `target/share/temper/bundle` so that `target/debug/temper`'s init
  finds it is not read any more.
- **The documents, when this lands:** `client/browser.md` 1, 2.2 and 5;
  `client/README.md`, section 8 ("when the configuration names a
  bundle"); `protocol/configuration.md` 3 (the `bundle` row);
  `protocol/setup.md` 8.2; `protocol/README.md`'s crates table (the new
  crate); `domain/testing.md` (2.6, the process tier's box);
  `docs/development/workflow.md` (the gate's and the agents'
  `TEMPER_BUNDLE`); `docs/guide/box.md`, step 1; `STATUS.md`.

## 5. Testing

Every suite passes in both builds, and the gate runs them under `none`;
none of them needs the embed.

- **The build script's own logic,** the dep-info read into the files it
  tracks and the bundle's directory read into its list, as plain
  functions, tested on samples in the crate's tests, in both builds.
- **The binary's read of a bundle from files in memory,** the code the
  embedded list goes through at start, tested on fixture files: the
  `Bundle` it makes, its refusals past the limits and off the closed
  list, as a directory's read is tested.
- **A start with `bundle` `null` expects what the binary holds,** which
  `temper`'s crate says: the page for no bundle under `none`, and
  otherwise the bundle's page at `/` and a file under
  `/bundle/<digest>/`. So `qa_serving`'s cases of a start with no
  bundle, and the process tier's box on empty directories
  (`protocol/setup.md`, 13.4; `crates/temper/tests/process.rs`,
  `assert_demo_streams`, init's line among the box's lines), each check
  the build they run in. The page for no bundle stays proved where it
  is built, `temper-service`'s `tests/bundle.rs`.
- **The embed itself is a person's build,** which fails loudly when its
  steps do. A change to the embed checks it with `TEMPER_BUNDLE=embed
  heavy cargo nextest run -p temper`, the same suites over the binary's
  bundle; the gate does not.
- **The browser tier** is unchanged: it names its own bundle.

## 6. What a person's build pays

Measured on 2026-10-23 on the development machine, through `heavy`,
with the spike's crate as a dependency of `temper` (its build script
kept in `~/src/rust/tmp/bundle-in-binary-spike/`, not versioned), as
evidence for the general claims of section 1; the plan measures them
again on its build. The spike called the bundler's library from its
script, with three of its crates optimized, where section 2 builds the
bundler whole, optimized, and runs it: its bundling took 1.9 to 2.4 s,
and 1.0 s with the bundler built so.

| What | Cost |
|---|---|
| The dev bundle | 9.0 MB in 14 files, 8.8 MB of it the module; the debug binary, 196 MB, grows by as much |
| The first run, from nothing | the module 40.8 s; the bundler's release build, measured apart, 56 s; bundling, 1 s |
| An edit outside the client's crates | nothing: the script does not run (0.13 s for its crate) |
| An edit to `temper-client-view` (a comment, the cheapest), then a build of `temper` | 7.4 s, against 1.3 s without the embed: the module 1.5 to 3.0 s, bundling, and `temper` built again with the new bytes |
| `cargo clippy` after a build, nothing changed | its own run of the script, once: 0.16 s for the nested build, which finds the module built, then bundling (2.2) |
| A build with `TEMPER_BUNDLE=none` | not measured: the build script of `std` alone, compiled and run once per target directory |

- **Builds that do not include `temper` pay nothing,** with or without
  the embed: a crate's tests (`heavy cargo nextest run -p
  temper-client-view`, and every domain's) never run the script.
- **The release bundle,** not measured, is expected smaller and is
  slower to make: `web`'s fat LTO over the module, once per release
  build that follows a client edit.

## 7. Later

- **A bundle built apart, embedded:** `TEMPER_BUNDLE=DIR`, a bundle
  directory to embed in place of building one, so that one wasm32 build
  serves binaries for several hosts, and a machine without the target
  can still make a whole binary. Waits for the release pipeline.
- **Served from the binary's bytes,** with no copy at start: a `Bundle`
  that borrows. Waits for a bundle whose copy matters.
- **skein's, for every application:** the build script's steps (the
  nested build, its directory, its environment, what reruns it) are
  nothing of temper's. They move to a small crate of skein's that a
  build script depends on, when a second application embeds its bundle
  (jig's reference application, or skein's example), as skein's
  `ui-web.md`, section 15 has it.
- **A binary to download:** where release binaries are built, for which
  targets and C library (a static musl build, or a floor of glibc), and
  where they are published. This draft makes the binary whole; it does
  not ship it.

## 8. Open questions

1. **The embed without the wasm32 target:** the build fails, naming the
   two ways out (2.4), or it warns and makes a binary without the
   browser. Default: it fails. A silent binary without its browser is
   what this draft removes, and the machine's builds, which set `none`,
   never meet it.
2. **A `rust-toolchain.toml` naming the target,** so that rustup adds it
   by itself on a first build. It also sets the channel for every
   rustup user, which temper does not set today (`rust-version` alone).
   Default: none; the failure's message names the command.
3. **A switch to serve no browser** from a binary that has one, such as
   `clients.bundle` `false`. Default: none, until a deployment asks; the
   browser is reached only by a printed link, as the terminal is.
