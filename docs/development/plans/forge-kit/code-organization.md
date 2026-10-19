# forge-kit: code organization

What moves where, the names the session files use, who owns what, the
fixtures, the "Don't" list, and how a move is counted. The design's crate
map is skein's `forge.md`, 2.1; the findings give each file's lines at the
base (`findings/survey-0*.md`) and the design's names
(`findings/trace-*.md`, section 3 of each).

## 1. What moves where

| From (temper, at the base) | To (skein) | Step |
|---|---|---|
| `crates/temper-engine-domain-forge-client/src/api.rs`'s vocabulary (lines 1 to about 417; `answer_bytes_unbounded` stays the client's until skein 02.2) and `locate.rs` (`bytes`, `accepts` made `pub`) | `crates/skein-forge/src/{api.rs, locate.rs}` | skein 01.1 |
| `crates/temper-forge-forgejo` (the whole) | `crates/skein-forge/src/forgejo/` | skein 01.2 |
| `crates/temper-engine-protocol-forge/src/`: the call plans (`reads`, `reads/done`, `reads/size`, `reading`, `writes`, `contents`, `actions`, `plan`, `tables`, `revision`), `failure`'s classification, `hooks/hint`, keys' markers (`marker.rs`), and `Heard` and `Body` from `connection.rs` | `crates/skein-forge/src/forgejo/plans/`, `crates/skein-forge/src/marker.rs` | skein 01.3 |
| the component's start reads (`component.rs`'s `start_request`, `drive_starts`; `start.rs`'s `Heard`), and `conventions::host` | the dialect's start plans, skein-forge's pure `host(address, name)`; credentials' kinds, new | skein 01.4 |
| the vocabulary's measure (`answer_bytes`, `answer_bytes_unbounded`, the client's `bounds.rs` widths) | `skein_forge::measure` (`op`, `answer`) and the client's `measure` (`stored`, `effect`, the rows check), one increment (decision 24) | skein 02.2 |
| `crates/temper-engine-domain-forge-client` (the rest, with `api.rs`'s answer check, `accepts`) | `crates/skein-forge-client` | skein 02.1 |
| `crates/temper-engine-protocol-forge` (the rest: the component, the listener, `conventions`' user agent made the owner's, the 68 plan tests that drive a `Component`) | `crates/skein-forge-connection` | skein 03.1 |
| `tests/engine/protocol-forge`'s machine and listener parts (`tests/{machine, hooks, fuzzy_*}.rs`, `src/{machine, hooks, peer, pipe}`) | `tests/forge/machine` | skein 08.1 |
| `crates/temper-forge-setup/src/`: `reconcile.rs`, `reconcile/{hooks, orgs, tokens}`, the limits' check and the validators | `crates/skein-forge-setup` | skein 04.1 |
| `testing/temper-fake-forge-domain`, and `tests/fake-forge`'s memory tests | `testing/skein-fake-forge-domain`, its own `tests/memory.rs` | skein 05.1 |
| `testing/temper-fake-forge-protocol` | `testing/skein-fake-forge-protocol` | skein 05.2 |
| `tests/one-process/src/forge.rs` (`ForgeHost`), `temper-fake-git`'s shared forge handle | `testing/skein-fake-peers/src/forge/` | skein 05.4 |
| `testing/temper-throwaway-forgejo` | `testing/skein-throwaway-forgejo` | skein 06.1 |
| `tools/forgejo-conformance`, with its transcripts, deliveries and `facts.txt` | `tests/forge/conformance` | skein 06.2 |
| `tests/codecs`' dialect tests (`forgejo_*`, 55 focused and 3 fuzzy) | `crates/skein-forge/tests/` | skein 06.3 |
| `tests/engine/forge/client` | `tests/forge/client` | skein 07.1 |
| `tests/forge-setup/reconcile` | `tests/forge/setup` | skein 07.3 |
| `tests/engine/protocol-forge` (the rest: the component's and the fakes' tests; the tests of temper's connector and engines stay temper's, decision 22) | `tests/forge/protocol` | skein 09.1 |

**Stays temper's** (skein's `forge.md` 2.3): the forge connector and
`EffectPurpose`; the protocol layer's translation and the store's
records; the configuration and its files; `temper-forge-setup`'s setup,
check, problem, state, place, ini, runner, spec, prepare, run, upgrade,
step, the limits' `apply` and the `setup.*` names, and all of
`temper-forge-setup-shell`; `temper-fake-git` (its forge handle aside);
`tests/codecs` but its dialect tests; `tools/git-transcripts`, on skein's
throwaway; the one-process world and its stories, on skein's forge peer
after temper 05.2.

**skein's workspace:** each new crate under `crates/` joins by the
`crates/*` glob; each new `testing/` and `tests/` crate is listed by
hand in skein's `Cargo.toml` members, as skein's other test crates are.
skein-forge depends on skein-lib and skein-json; the client on skein-lib
and skein-forge; the component on skein-lib, skein-io, skein-http,
skein-tls, skein-json and skein-forge; setup on skein-lib, skein-json and
skein-forge (`forge.md`, 2.1). The client and the component don't
depend on each other: temper-engine-protocol-forge's use of the client
(`limits::Shared`'s client numbers) becomes plain numbers in skein 03.1.
There is no serde: skein-json takes its place (skein's dependency
allowlist).

## 2. Modules and names

The names are the design's, as each trace lists them
(`findings/trace-forge.md`, `trace-client.md`, `trace-connection.md`,
`trace-setup.md`, `trace-fakes.md`, section 3), checked with `rg` at the
base. A moved module keeps its file names under its new crate. A moved
test keeps its name. A name that names temper takes the design's
neutral name; the trace lists each such name.

## 3. Shared interfaces and their owners

| Interface | Owner (the step that makes it) | Users |
|---|---|---|
| `skein_forge::api`, `locate` (the vocabulary) | skein 01.1 | the client, the component, the plans, temper's connector and protocol layer |
| `skein_forge::forgejo` (documents, deliveries, the setup's documents, the pin) | skein 01.2 | the plans, setup, the fakes, the throwaway, the conformance |
| `skein_forge::forgejo::plans` (`begin`, `heard`, `Heard`, `Body`, the bounds) | skein 01.3 | the component, the conformance |
| `skein_forge::marker` (the owner's word) | skein 01.3 | the plans, the client's echo, setup's bot marker |
| credentials' kinds, the start plans, `host` | skein 01.4 | the component, setup, temper-service |
| `skein_forge::measure`, the client's `measure` | skein 02.2 | the client, the component's page size and 7.4 check, temper's connector |
| `skein_forge_client` | skein 02.1 | temper's domain and connector |
| `skein_forge_connection` (`Component`, `Settings`, `Limits`, `Unusable`) | skein 03.1 | temper-service, the conformance, the worlds |
| `skein_forge_setup` (the machine, the given values, `Item`, `Place`) | skein 04.1 | temper-forge-setup, temper-forge-setup-shell, temper-box |
| `skein_fake_forge_domain`, `skein_fake_forge_protocol` | skein 05.1, 05.2 | the peer, the worlds, temper's worlds |
| `skein_fake_peers::forge` (`Peer`) | skein 05.4, its delivery 05.5 | temper's one-process world, skein's worlds |
| `skein_throwaway_forgejo` | skein 06.1 | the conformance, temper's process tier and `tools/git-transcripts` |

An interface changes only in its owner's later increments, or in an
increment that names the change in its Shape.

## 4. Checks the plan adds

None to the gates. README "Done when" 5 checks that skein's kit names no
system.

## 5. Fixtures

- **The owner's values in skein's tests:** the word `kit`, the user
  agent `kit/0.0.0`, the namespace `b"kit"`; setup's admin `kit-admin`,
  bot `kit:bot`, tokens `kit-api` and `kit-git`. A test whose bytes held
  temper's word holds `kit`'s, its lengths derived from the word's
  length (a named size).
- **temper's values,** handed by temper's owner code: the word `temper`,
  the user agent `temper/0.0.0`, the namespace the deployment's, setup's
  `temper-admin`, `temper:bot`, `temper-api`, `temper-git`. temper 01.3
  adds a test that pins `<!-- temper:key <hex> -->\n`, and temper 03.1
  one that pins `temper/0.0.0`.
- **The fake protocol layer's fixtures** are its own (today it borrows
  `temper-engine-forge-world`'s `fake_config`): a `fixtures.rs` in its
  `tests/`.
- **The transcripts** stay byte for byte, in `tests/forge/conformance/`
  from skein 06.2. Only the conformance's recording writes them.

## 6. A move, and its count

A move copies each file from temper's `origin/main` at the step's start
(the "from" commit, named in the commit body) to its skein path, then
renames paths (`temper_forge_forgejo::` to `skein_forge::forgejo::`, and
so on) and drops temper's names. The count, per file pair, from the
skein worktree:

```sh
git diff --no-index --numstat <temper worktree>/<from file> <to file>
```

The added lines of each pair are the step's new lines in that file, and
the file's other lines are moved. A file split out of a base file
pairs with that file, so its lines from it count as moved; a file with
no base source is all new. Tests pair the same way: a moved test's
lines are moved. A Size counts production lines, as development.md
5.2's template says, and says so where it counts tests too; an
increment of tests alone counts its tests. A mechanical rewrite in
place in temper (an import, a path, an accessor's spelling, each named
in Shape) counts as moved, as temper 02.1's callers and temper 06.3's
reads do. The increment's report
gives the sums, production and tests apart, with the file list. The review reads
the added lines alone.

## 7. The plan's "Don't" list

- No rewrite of a moved part: an increment copies files and renames
  paths, and changes besides only what its new home forces, each named
  in its Shape. A behaviour change is a later increment's, with its own
  test.
- No name of temper, or of any service, in skein's code, comments,
  tests or fixtures; no re-export or alias of a moved type, in skein or
  in temper: temper's users name skein's paths.
- No dependency between `skein-forge-client` and
  `skein-forge-connection`; no serde, and no dependency outside skein's
  allowlist.
- No change to a hotspot file outside the steps README's Rules name.
- No budget raised, no test filtered, ignored or split to meet a
  ceiling, no optimized test build.
- No `Cargo.lock` merged by hand.
- No transcript, delivery or `facts.txt` edited by hand.

## 8. Size limits

A file of at most 800 lines (the pulse's `file-lines`) and a test
function of at most 40 lines, an aim. The moved files keep their size:
a file over 800 lines that moves is split by the increment that next
changes it (`keep.rs` 1377, `outbox.rs` 1079 and `api.rs` 850 at the
base; skein 02.2 splits the first two; `api.rs` splits by its move).
