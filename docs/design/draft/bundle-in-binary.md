# The browser in the binary

Draft, 2026-10-23. temper's binary carries the browser's bundle, so that
`temper box` and `temper serve` serve the browser with nothing beside
the binary, and `cargo build` in a checkout makes that binary. It
answers `client/browser.md`'s open question "The bundle in the binary"
(section 5), whose default this draft replaces. A spike, a throwaway
crate in a worktree, never landed, built the dev bundle inside `cargo
build` as section 2 says; section 6 holds its numbers. Section 8 holds
what is still open, each with a default.

## 1. In one page

- **One file.** The `temper` binary holds the bundle: the shell's
  module, its glue and the stylesheets and files skein's bundler puts
  beside them (`client/browser.md`, 2.2 and 2.5). Running it is the
  whole install: no `share/temper/bundle`, no `scripts/install.sh`
  step, no `scripts/bundle.sh`.
- **`cargo build` makes it.** `cargo build`, then `./target/debug/temper
  box`, serves the browser. So does `cargo build --release`, and `cargo
  install --git` of temper's repository.
- **A crate of its own embeds it,** `temper-client-bundle`, which
  `temper` depends on. Its build script builds `temper-client-shell-web`
  for wasm32 with a nested cargo, in a target directory of its own,
  runs skein's bundler's library on the module, and embeds the bundle's
  files (section 2). No artifact dependency, no `xtask`, nothing
  unstable: a build script running cargo for another target is the
  pattern substrate's `wasm-builder` and risc0's `risc0-build` ship.
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
- **It needs the wasm32 target,** as temper's gate already does. A
  build without it fails, naming the two ways out: add the target, or
  build without the browser, `TEMPER_BUNDLE=none` (2.4).
- **What it costs** (section 6, measured): once, the script's first
  run, about a minute and a half, 40 s of it the module; then, after an edit to the client's crates,
  about 6 s more on a build of `temper` (the module's incremental build,
  bundling, and `temper` built again with the new bytes); nothing on a
  build that does not include `temper`, or after any other edit.

## 2. The build

### 2.1 The crate

`crates/temper-client-bundle`: ordinary Rust, as skein's bundler is
(skein's `ui.md`, section 2), with its own `[lints]` table, as tools
have.

- **Its build script:**
  1. builds `temper-client-shell-web` with the cargo running it
     (`CARGO`): `build --locked --target wasm32-unknown-unknown -p
     temper-client-shell-web`, with the profile of 2.3, from the
     workspace's root, into the target directory of 2.2;
  2. runs `skein-ui-bundle`'s library, its `glue` then its `bundle`,
     with the inputs `scripts/bundle.sh` gives the bundler's binary
     today (the name, the prefix `/bundle/`, `temper.css`,
     `task-pages.css`, the shell's `files/` and the noscript words),
     into its `OUT_DIR`;
  3. writes, into `OUT_DIR`, the list of the bundle's files, each its
     path and its bytes by `include_bytes!`, which the crate's library
     includes.
- **Its library** is that list and the bundle's digest: data, no code
  but the include. The binary's crate reads it (section 3). With
  `TEMPER_BUNDLE=none`, the list is empty and the binary has no bundle.
- **`skein-ui-bundle` is its build dependency,** never a dependency of
  the binary: it is compiled for the build script and not linked into
  `temper`, as skein's `ui.md` has it ("never linked into a service or a
  module"). One `Cargo.lock` still locks the bundler's
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
  test build of `temper` reuse one module, and the build after a change
  is incremental.
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
- **The bundler optimized in dev,** `skein-ui-bundle` and the
  `wasm-bindgen` crates it calls, by `[profile.dev.package.…]`, as skein
  builds `skein-ui-bundle` and temper builds `skein-codegen`: on today's
  dev module, bundling takes 5.3 s (12 s of CPU) with none optimized,
  1.9 s in the spike's script with three of them (`skein-ui-bundle`,
  `wasm-bindgen-cli-support`, `walrus`), and 1.0 s with all of them.
  Which, exactly, is the plan's measure.

### 2.4 Without the wasm32 target, and without the browser

- **No target:** the nested build fails, and the script fails the
  build with one message: `temper-client-bundle: the browser's bundle
  needs the wasm32-unknown-unknown target: add it (rustup target add
  wasm32-unknown-unknown), or build without the browser
  (TEMPER_BUNDLE=none)`. Never a binary without the browser in
  silence (8.1).
- **`TEMPER_BUNDLE=none`:** no nested build and no bundle; for a
  machine without the target, and for an editor's own builds
  (rust-analyzer runs build scripts at its start). Such a binary serves
  the page for no bundle, as today's binary does with no bundle
  configured (section 3).
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

## 4. What else changes

- **`scripts/install.sh`** installs the binary alone, built with
  `--release --locked`, and prints its path.
- **`scripts/bundle.sh`** stays, for a bundle built apart: the browser
  tier's, which turns on skein-ui-web's `loopback-navigation`
  (`client/browser.md`, 4.3) and so is never the binary's, and a
  developer's. It builds into the same `<target>/bundle/` directory, so
  the tier's module shares the embedded one's dependencies and builds
  only what the feature changes.
- **The gate** (`scripts/check.sh`) builds the embedded bundle within
  its first cargo command that builds `temper` (its clippy run), and
  the tier's as now. Both need the wasm32 target, which it already
  needs.
- **The browser-box workaround goes:** a bundle copied to
  `target/share/temper/bundle` so that `target/debug/temper`'s init
  finds it is not read any more.
- **The documents, when this lands:** `client/browser.md` 1, 2.2 and 5;
  `client/README.md`, section 8 ("when the configuration names a
  bundle"); `protocol/configuration.md` 3 (the `bundle` row);
  `protocol/setup.md` 8.2; `protocol/README.md`'s crates table (the new
  crate); `domain/testing.md` (2.6, the process tier's box);
  `docs/guide/box.md`, step 1; `STATUS.md`.

## 5. Testing

- **The build script's own logic,** the dep-info read into the files it
  tracks and the bundle's directory read into its list, as plain
  functions, tested on samples in the crate's tests. The nested build
  itself is proved by every build that embeds, the gate's among them.
- **The binary's crate,** on the embedded bundle: it becomes a `Bundle`
  within the default profile's limits, its page naming its digest's
  directory, every file typed. The dev bundle is the larger, so a
  module that outgrows `clients.bundle_bytes`' default (16 MiB, nearly
  twice today's) fails this test, never a person's start.
- **`qa_serving`:** with `bundle` `null`, the binary's bundle is served:
  the page at `/` and a file under `/bundle/<digest>/`. The page for no
  bundle stays proved where it is built, `temper-service`'s
  `tests/bundle.rs`; `qa_serving`'s cases of a start with no bundle
  become cases of `null`.
- **The process tier:** the box on empty directories
  (`protocol/setup.md`, 13.4) answers `GET /` with the bundle's page,
  not the page for no bundle, and its stderr holds no `init: no bundle`
  line (`crates/temper/tests/process.rs`, `assert_demo_streams`).
- **The browser tier** is unchanged: it names its own bundle.

## 6. What it costs

Measured on 2026-10-23 on the development machine, through `heavy`,
with the spike's crate as a dependency of `temper` (its build script
kept in `~/src/rust/tmp/bundle-in-binary-spike/`, not versioned), as
evidence for the general claims of section 1; the plan measures them
again on its build.

| What | Cost |
|---|---|
| The dev bundle | 9.0 MB in 14 files, 8.8 MB of it the module; the debug binary, 196 MB, grows by as much |
| The script's first run, from nothing | 1 min 28 s for its crate alone: 40.8 s the nested build, 2.1 s bundling, the rest the bundler's own build for the script |
| An edit outside the client's crates | nothing: the script does not run (0.13 s for its crate) |
| An edit to `temper-client-view` (a comment, the cheapest), then a build of `temper` | 7.4 s, against 1.3 s without the embed: the module 1.5 to 3.0 s, bundling 1.9 to 2.4 s, and `temper` built again with the new bytes |
| `cargo clippy` after a build, nothing changed | its own run of the script, once: 0.16 s for the nested build, which finds the module built, and 1.9 s bundling (2.2) |

- **Only builds of `temper` pay.** A crate's tests that do not build
  `temper` (`heavy cargo nextest run -p temper-client-view`, and every
  domain's) never run the script. Builds that do, `temper`'s own tests
  and the real loop's world, which depends on `temper`, pay once after
  each client edit.
- **Bundling is most of it,** and the bundler's crates all optimized
  take it from 1.9 s to 1.0 s (2.3).
- **The release bundle,** not measured, is expected smaller and is
  slower to make: `web`'s fat LTO
  over the module, once per release build that follows a client edit.

## 7. Later

- **A bundle built apart, embedded:** `TEMPER_BUNDLE=DIR`, a bundle
  directory to embed in place of building one, so that one wasm32 build
  serves binaries for several hosts, and a machine without the target
  can still make a whole binary. Waits for the release pipeline.
- **Served from the binary's bytes,** with no copy at start: a `Bundle`
  that borrows. Waits for a bundle whose copy matters.
- **skein's, for every application:** the build script's steps (the
  nested build, its directory, its environment, what reruns it) are
  nothing of temper's. They move to `skein-ui-bundle`'s library, as a
  function a build script calls, when a second application embeds its
  bundle (jig's reference application, or skein's example), as skein's
  `ui-web.md`, section 15 has it.
- **A binary to download:** where release binaries are built, for which
  targets and C library (a static musl build, or a floor of glibc), and
  where they are published. This draft makes the binary whole; it does
  not ship it.

## 8. Open questions

1. **Without the wasm32 target:** the build fails, naming the two ways
   out (2.4), or it warns and makes a binary without the browser.
   Default: it fails. A silent binary without its browser is what this
   draft removes, and every machine that runs temper's gate already has
   the target.
2. **A `rust-toolchain.toml` naming the target,** so that rustup adds it
   by itself on a first build. It also sets the channel for every
   rustup user, which temper does not set today (`rust-version` alone).
   Default: none; the failure's message names the command.
3. **A switch to serve no browser** from a binary that has one, such as
   `clients.bundle` `false`. Default: none, until a deployment asks; the
   browser is reached only by a printed link, as the terminal is.
