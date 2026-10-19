# Survey 07: dependency pins (read only; nothing built, fetched or run)
Read at: skein origin/main b4cc6eb, smith a277e92, jig b2d920e, hearth 5609614, temper 27995fb. Skein's and jig's checked-out HEAD (d120820, 62bcf71 = github/main) is one docs-only commit past origin/main; all counts use origin/main.

## (a) How each repository depends upstream
```
$ grep -E 'git = ' <repo>/Cargo.toml | sed -E 's/.*forgetic\/([a-z]+)\.git.*/\1/' | sort | uniq -c
$ grep -o 'git+https://github.com/forgetic/[a-z]*\.git[^"]*' <repo>/Cargo.lock | sort | uniq -c
```

| repo | Cargo.toml (all `{ git = "https://github.com/forgetic/<r>.git" }`, no branch/rev/tag) | Cargo.lock: one revision per source |
|---|---|---|
| temper | 85 deps: 45 jig, 23 skein, 17 smith | skein 8a51e10b873f (29 pkgs), smith 29fa97c9cdfd (28), jig bbea94081ae0 (52) |
| jig | 31 skein, 14 smith | skein 0f093bb74454 (34), smith 29fa97c9cdfd (23) |
| smith | 25 skein | skein 841f4877dbf5 (25) |
| hearth | 8 skein | skein fea416ff30e6 (8) |

Each lock holds exactly one revision per git source (no `?branch=`; temper's has one `skein-lib`). No `[patch]` or `[source]` anywhere. A dependency's own Cargo.lock is ignored: jig@bbea94081 locks skein 0f093bb and smith@29fa97c9 locks skein 74b7fda, but temper's lock holds only 8a51e10.

## (b) Skein pins versus skein origin/main
```
$ git -C skein rev-list --count <pin>..origin/main        # every pin is an ancestor of origin/main
$ git -C skein log --oneline --reverse <pin>..origin/main -- crates testing tests
```

| holder | pin | behind | commits touching crates/ testing/ tests/ |
|---|---|---|---|
| temper | 8a51e10 (2026-10-16) | 97 | 17 |
| jig | 0f093bb (parent of 8a51e10) | 98 | 18 (adds 8a51e10) |
| smith | 841f487 (2026-10-17) | 76 | 3: 05aa3c4 skein-world `spawn_restartable_startup`, 0b08745 sim test, 0e6f44e comment |
| hearth | fea416f (2026-10-18) | 49 | 1: 0e6f44e (a comment); the rest is scripts and docs: no API change |

API-bearing commits for temper and jig, oldest first (1 is jig only; temper's pin has it):
1. 8a51e10 Add streaming HMAC-SHA256 and constant-time byte comparison (skein-lib)
2. f981746 Use lib SHA-256 for PKCE and browser bundle names (skein-oauth; ui-bundle file names change)
3. 089ed83 Expose the first byte position of each JSON token (skein-json `Position`); 59b912d Locate JSON grammar failures and completed documents; b888da9 Place string failures at their character or escape's first byte
4. 24be04f Move file names between two open directories (skein-io; skein-kv temp-file names change)
5. 303c829 Add exclusive directory locks at the kernel boundary (`kernel::Op::Lock`, `Error::Busy`: BREAKING)
6. 2112fce List an owner's open directory in bounded pages (skein-io `ListPage`)
7. 3432cf5 Expose nonblocking root locks to file owners (skein-io `Lock`/`Locked`/`Busy`)
8. aaddf88 Add a target-independent SHA-256 state hasher (skein-lib `StateHasher`); 78dd8fa Hash lib containers by their held state
9. 6141021 Expose process and machine state digests at a world's end (skein-sim `StateDigests`)
10. 841f487 Trace process and machine digests after world iterations (skein-sim `Digests`)
11. 05aa3c4 Reopen startup append files for restartable world processes (skein-world)
12. 1eba6c1 Recover a moved directory with its surviving parent (skein-fake-machine)

Why 5 breaks: `kernel::Op`/`Error` are not `non_exhaustive` and clippy denies wildcard arms; 303c829 itself added `Op::Lock` arms to skein's echo tests. Downstream exhaustive `Op` matches without `Lock`:

```
$ grep -c 'Op::Lock' temper/crates/temper-client-shell-terminal/tests/terminal.rs         # 0
$ git -C smith show 29fa97c9:tests/agent-process/src/world.rs | grep -c 'Op::Lock'        # 0
$ git -C smith log -S'Op::Lock' origin/main -- tests/agent-process/src/world.rs           # b415f737
```

So a skein repin fails to compile temper's `terminal.rs` test and `smith-agent-process-world` (a smith test-world crate temper depends on) at smith 29fa97c9. Smith b415f737 "Repin skein and handle its new file operation variants" (191 commits after 29fa97c9) fixes the latter. jig@bbea94081 has no exhaustive `Op` match.

## (c) temper's pins of jig and smith
```
$ git -C jig rev-list --count bbea94081..origin/main     # 2
$ git -C smith rev-list --count 29fa97c9..origin/main    # 197
```

jig bbea94081 (2026-10-18) is 2 behind (f50c3f6 client named sends; b2d920e Mark client-api done). smith 29fa97c9 (2026-10-14) is 197 behind; jig's own lock pins the same smith, so temper's lock holds one smith for both. jig main still locks smith 29fa97c9 and skein 0f093bb: verified against neither newer smith nor newer skein.

## (d) temper members: forge, fake, git, setup, and worlds using forge crates
```
$ for m in <workspace members>; do grep -m1 '^name' $m/Cargo.toml; awk '[dependencies]/[dev-dependencies] temper-*'; done
```

Abbreviations (drop `temper-`): F4 = engine-domain-forge + -forge-change + -forge-client + -forge-issues; P = engine-protocol-forge; J = forge-forgejo; S = forge-setup; FD = fake-forge-domain; FP = fake-forge-protocol; TJ = throwaway-forgejo; EW = engine-forge-world; PW = engine-protocol-forge-world; FC = fake-checkout.

- crates/: the four engine-domain-forge* (forge depends on the other three; those depend on none); engine-protocol-forge (forge-client, J); forge-forgejo (leaf); forge-setup (J); forge-setup-shell (P, J, S); worker-protocol-git (leaf; git, not forge).
- testing/: fake-forge-domain (none); fake-forge-protocol (EW, FD, J); throwaway-forgejo (J); fake-git (FC, FD, FP, worker-protocol-git and its world).
- tests/fake-checkout = FC (none; skein also has skein-fake-checkout); tests/fake-forge (FD); tests/forge-setup/reconcile (PW, FP, J, S); tests/engine/forge = EW (F4, FD); tests/engine/forge/client (forge-client, FD); tests/engine/protocol-forge = PW (F4, EW, P, FD, FP, J); tests/engine/domain (forge, -change, -client, EW, FD); tests/system (same plus FC); tests/conformance (as domain); tests/codecs (F4, P, J); tests/client (forge, forge-client); tests/one-process (forge, P, PW, FC, FD, FP, fake-git); tests/worker/checkout (FC, FD); tests/worker/domain (FC); tests/worker/protocol-git (worker-protocol-git).
- tools/forgejo-conformance (forge-client, P, TJ); tools/git-transcripts (TJ).
- Non-forge consumers: engine-domain (F4), engine-smith (forge, forge-client), protocol-store (F4), service (F4, P, worker-protocol-git), temper bin (P, J, S, forge-setup-shell, TJ), temper-box (S).

## (e) Reverse dependencies and hearth
```
$ git -C <repo> ls-files '*Cargo.toml' Cargo.lock | xargs grep -il temper
$ git -C hearth grep -il forge -- '*.rs' '*.toml'
```

- No skein, smith, jig or hearth package or dependency is named temper*. Strings only: skein-fake-checkout's `.git/temper-head`, smith fixtures `work/temper/Cargo.toml`, smith-domain's description.
- hearth has no forge code: hits are `mem_forget` lints and a forgejo release URL in builder test fixtures. It has its own git layer (hearth-protocol-git, hearth-fake-git, tests/git). skein has no forge crate, only forgejo transcripts under tests/http and tests/json.

## (f) skein's workspace Cargo.toml
- `members`: glob `"crates/*"` (18 crates), then explicit paths: examples/ui/{view,shell-web,server}, examples/echo/{domain,protocol,service,shell}, 16 `testing/skein-*`, 24 `tests/*` (tests/conformance/{ring,sim} and flat tests/{browser,codec,...,world}). New testing/ and tests/ crates are added by hand.
- `resolver = "3"`; `[workspace.package]` 0.0.0, edition 2024, rust-version 1.95, publish false.
- `[workspace.dependencies]`: each skein crate by path; externals only wasm-bindgen(-cli-support) =0.2.126, js-sys, web-sys =0.3.103, io-uring 0.7.15, libc 0.2.186, rustls =0.23.41 (no default features; ring, tls12). serde is absent (in the lock only via wasm-bindgen-cli-support).
- Profiles: dev/release `panic = "abort"`, overflow-checks; rustls debug-assertions off and skein-ui-bundle opt-level 3 in dev; `profile.web`. Profiles come from the root only, so a consumer copies them.
- Lints: `[workspace.lints.rust]` (unsafe_code deny, missing_debug_implementations deny, ...) and `[workspace.lints.clippy]` (pedantic warn plus the programming-model 10.3 subset). A step crate inherits with `[lints] workspace = true`, held by the root clippy.toml (bans Vec, String, HashMap, closures). Ordinary-Rust crates (testing/*, tests/*, shell, sim) write their own `[lints.rust]/[lints.clippy]` plus a clippy.toml (20 exist; tests/clippy.toml covers tests/*). temper's root repeats the same tables.

## (g) Repinning temper onto skein main
```
$ cargo update -p skein-lib      # not run; read from the locks
```

The lock shows one skein source (29 packages, one `skein-lib`) shared by temper, jig@bbea94081 and smith@29fa97c9: their manifests name the same bare URL, so cargo unifies them and updating that source moves every skein-* package at once to b4cc6eb. Yes: jig's and smith's crates in temper's build resolve to the same skein.

It does not compile alone: they are built unchanged against the new skein, and `smith-agent-process-world` @29fa97c9 and temper's `terminal.rs` lack the `Op::Lock` arm (see (b)). The first skein repin needs a smith repin to >= b415f737 (main a277e92 is 197 ahead; jig is unverified against it) and the arm in temper. Whether jig@bbea94081 builds against skein main only a real build settles.
