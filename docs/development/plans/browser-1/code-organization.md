# browser-1: code organization

## 1. Crates

skein adds no crate. It changes:

| Crate | Kind | What changes | Session |
|---|---|---|---|
| `testing/skein-fake-person-browser` | test kit | the steps `Step::Address`, which reads the page's address, and `Step::KeyWithShift`; `back_and_forward` takes a `loaded` find | skein 01.1 to 01.3 |
| `testing/skein-browser` | test kit | `Request::Address` and `Event::Address`; `Request::Key`'s `shift` (`ui-web.md`, 14.2) | skein 01.1, 01.3 |
| `crates/skein-ui-web` | library | the click listener finds the nearest link, button or menu item (`ui-web.md`, 6) | skein 01.4 |
| `tests/browser` | world | the kit's `Address` and Shift through `World`, and the referee's terminals | skein 01.1, 01.3 |
| `examples/ui/view`, `tests/ui-web` | example, world | a node the click test presses inside; its goldens; the browser tests' callers of `Step::Address` and `back_and_forward` | skein 01.1, 01.2, 01.4 |

temper adds two crates:

| Crate | Kind | What it holds | Session |
|---|---|---|---|
| `crates/temper-client-shell-web` | `cdylib`, every dependency under `cfg(target_arch = "wasm32")`, as jig's `jig-client-test-shell-web` | `lib.rs` (start), `host.rs`, `records.rs`, `limits.rs`; `temper.css`; `files/robots.txt` | temper 02.2 to 02.4, 03.3 |
| `tests/client-browser`, package `temper-client-browser-world` | world | `src/lib.rs`: the tier's setup, as jig's `tests/client-browser`; `tests/browser_sign_in.rs`, `tests/browser_pages.rs`, `tests/browser_attacks.rs` | temper 04 |

and changes:

| Crate | What changes | Session |
|---|---|---|
| `temper-service` | `application.clients.bundle`, the two limits, start's checks; the bundle's answers and routes in `src/clients/bundle.rs`; the page policy in `src/clients/policy.rs` | temper 01 |
| `temper` (the binary) | `src/bundle.rs`: reading the bundle's directory at start, since `temper-service` is `no_std` | temper 01.1 |
| `temper-client-domain` | the address codec, in `src/address.rs`; `Owner::asker` becomes a `const fn` | temper 02.1; 03.3 |
| `temper-client-view` | `words::Shell` and the browser's signed-out words; the composers' SubmitsOnEnter and line breaks; the look's tokens and `NAMES`, in `src/look.rs` | temper 03 |
| `temper-client-shell-terminal` | `native()`'s header count, 8 to 16, for the policy's fields; `Words::english`'s shell argument | temper 01.3; 03.1 |
| `temper-client-shell-cli` | `native()`'s header count, 8 to 16, as the terminal's: `temper ctl` reads `/jig` | temper 01.3 |

`scripts/bundle.sh OUT` (temper 02.3) builds the shell for wasm32 with
the dev profile and runs skein's `skein-ui-bundle` into `OUT`, with
`skein-ui.css`, then `temper.css`, `files/robots.txt`, the prefix
`/bundle/` and the noscript words of decisions.md. `--release` builds
the release profile.

## 2. Names and public API

- Paths, routes and printed words come from the design by section;
  a session never names a new path.
- The codec: `temper_client_domain::address::{parse, print}` over the
  existing `Address` and `Place` (temper 02.1). No other crate parses
  or prints a path.
- The style tokens: constants of `temper-client-view`, each a `temper-`
  name (temper 03.3); `temper.css` names no class the view does not set.
- The tier: `temper_client_browser_world::Tier` and its `setup`, as jig's, and the bundle's
  variable `TEMPER_TEST_BUNDLE` (temper 04.1).
- skein's fake person: `Step::Address` and `Step::KeyWithShift` (skein
  01.1, 01.3) are public items of its harness, documented as its other
  steps. temper's tier names `Key` through a `skein-browser`
  dependency, since the fake person does not re-export it.

## 3. Owners of shared interfaces

| Interface | Owner | Users |
|---|---|---|
| `Step::Address` | skein 01.1 | temper 04.1 to 04.4 |
| `back_and_forward(…, loaded)` | skein 01.2 | temper 04.3 |
| `Step::KeyWithShift` | skein 01.3 | temper 04.4 |
| `application.clients.bundle`, `clients.bundle_files`, `clients.bundle_bytes` | temper 01.1 | 01.2, 04.1 |
| `temper::bundle::read` | temper 01.1 | the binary's start |
| the header count of the clients' `native()` limits | temper 01.3 | the terminal shell and the CLI, each its own copy; 02.2's shell copies only `client` and `view` |
| `words::Shell`, `Words::english(…, shell)` | temper 03.1 | the terminal shell; 02.2's shell, whichever of 02.2 and 03.1 lands second passing `Shell::Browser` |
| `look::NAMES` | temper 03.3 | the shell's look |
| the bundle's answers (`clients/bundle.rs`) | temper 01.2 | 01.3 |
| `address::{parse, print}` | temper 02.1 | 02.2, 02.4 |
| `temper.css` | temper 02.3 | 03.3 |
| the style tokens | temper 03.3 | `temper.css` |
| `scripts/bundle.sh OUT` | temper 02.3 | 04.1, the gate |
| `Tier::setup`, `Tier::states` and `TEMPER_TEST_BUNDLE` | temper 04.1 | 04.2 to 04.5 |
| `start_chat`, `FIRST_CHAT`, `FIRST_CHAT_ADDRESS` | temper 04.2 | 04.3, 04.4 |

## 4. Tests and fixtures

- Each crate's fixtures stay in its one module, extended, never
  redefined: `temper-service`'s `src/tests/fixtures.rs` and
  `src/tests/fixtures/`; `temper-client-domain`'s and
  `temper-client-view`'s `src/tests/fixtures.rs` (made by the first
  session that needs one there); the tier's in its `src/lib.rs`.
- The configuration's new key goes into every configuration the
  repository holds, in temper 01.1: `docs/examples/owned.json`,
  `crates/temper/tests/support/configuration.rs`,
  `tests/one-process/src/fixtures.rs`,
  `crates/temper-service/src/tests/fixtures/startup.rs` and
  `crates/temper-service/src/tests/fixtures.rs`, and every other user
  that session's search finds.
- The browser tier's binaries are named `browser_*`. The default profile
  excludes them (the fuzzy profile takes only `fuzzy_*`), and the `browser` profile takes only
  them, with one thread (temper 04.1).

## 5. Don't

- No type beside an existing one for the same thing: change the type in
  place, and its users in the same increment.
- No field or predicate that its counterpart across a boundary lacks.
- No test that breaks a rule of a test's shape (testing-strategy.md,
  section 10).
- No lint exception in tests.
- No word the Pulse forbids, in code.
- No browser test beyond the ten of decisions.md's "For the user", and
  no `#[ignore]` or early return when Chromium or the bundle is missing:
  the suite fails.
- No dependency of jig's test service or test shell: their policy,
  bundle serving and start are copied, as `browser.md`, 2.1 and 2.2, say.
- No host dependency on `wasm-bindgen`, `web-sys` or `js-sys`: the shell's
  are all under `cfg(target_arch = "wasm32")`.
- No change to the Chat page's turns, which live text streams into.
- No filter in the address; no path under `/jig`, `/bundle/` or
  `/test/`, nor `/robots.txt`, printed by the codec.
- No web font, icon or external origin in `temper.css`.
- No peer in the browser tier (decisions.md, Design questions 1).

## 6. Size limits

A file of at most 800 lines and a test function of at most 40.
`crates/temper-service/src/clients.rs` holds 641 lines at the base:
temper 01.2 puts the bundle's answers and their routing in
`src/clients/bundle.rs`, and 01.3 the policy in `src/clients/policy.rs`,
so that `clients.rs` stays under 650. `src/startup/application.rs` holds
708: temper 01.1 adds the key and nothing else there.
`crates/temper-service/src/tests/fixtures.rs` holds 841 at the base: the
bundle's new fixtures go in `src/tests/fixtures/bundle.rs`, and that
file gains only the configuration's `"bundle": null`.

skein's `testing/skein-browser/src/browser.rs` (2041 lines) and
`testing/skein-fake-person-browser/src/harness.rs` (822) are past the
limit at the base. skein 01.1 and 01.3 add their requests' arms to them,
as their files say, and split neither: the limit holds for every other
file, and a split is no part of this design.

An increment's size, in hand-written production lines, from its
worktree:

```
git diff --numstat origin/main... -- crates scripts ':(glob,exclude)**/tests/**' ':(glob,exclude)**/tests.rs' ':(glob,exclude)**/golden/**' | awk '{a += $1} END {print a + 0}'
```

in temper, and in skein:

```
git diff --numstat origin/main... -- crates testing examples ':(glob,exclude)**/tests/**' ':(glob,exclude)**/tests.rs' ':(glob,exclude)**/golden/**' | awk '{a += $1} END {print a + 0}'
```

The browser tier's crate, `tests/client-browser`, is a world: its lines
count as tests.
