# The renames at the base (step 1), and the naming rule's check

The code audit of jig at `a3b53546`, 2026-10-15, by a read-only
sub-agent, from `git grep` and `cargo metadata` (not the graph).

## 1. What each rename touches

Files naming the old crate in jig (Cargo.toml / .rs / other; "other" is
`Cargo.lock` for each), then in temper (Cargo.toml / .rs):

| Old directory | New name | jig | temper |
|---|---|---|---|
| `crates/jig-protocol-client-wire` | `jig-client-protocol-wire` | 11/63/1 | 5/11, and a schema comment |
| `crates/jig-protocol-client-link` | `jig-client-protocol-link` | 5/12/1 | 3/9 |
| `crates/jig-shell-native` | `jig-client-shell-native` | 5/9/1 | 4/8 |
| `crates/jig-view` | `jig-client-view` | 7/36/1 | 6/28 |
| `testing/jig-scripted-engine` | `jig-client-scripted-engine` | 4/28/1 | 0/0 |
| `testing/jig-test-domain-client` | `jig-client-test-domain` | 7/63/1 | 2/1 |
| `testing/jig-test-view` | `jig-client-test-view` | 4/13/1 | 0/0 |
| `testing/jig-test-client-kit` | `jig-client-test-kit` | 3/3/1 | 0/0 |
| `testing/jig-test-client-records` | `jig-client-test-records` | 5/17/1 | 0/0 |
| `testing/jig-test-protocol-client-wire` | `jig-client-test-protocol-wire` | 5/5/1 | 0/0 |
| `examples/ops/domain-client` | `jig-ops-client-domain` | 4/20/1, 2 directory lines | 0/0 |
| `examples/ops/view` | `jig-ops-client-view` | 3/3/1, 2 directory lines | 0/0 |

Together in jig: 23 manifests (71 lines), 177 `.rs` files (791 lines),
`Cargo.lock` (51 lines); in temper: 8 manifests (21 lines), 42 `.rs`
files (182 lines). temper still names `jig-fake-person`, which jig no
longer has (skein-ui).

## 2. Directories and the boundary's tests

- The directories move with the packages (jig-protocol 01's rule; the
  design's `examples.md`, section 2 names `examples/ops/client-domain`
  and `examples/ops/client-view`): `crates/jig-client-*`,
  `testing/jig-client-*`; the worlds keep `tests/client*`.
- `tests/boundary` names them in `src/tests.rs` 238-246, 269-297,
  340-352 and `src/tests/graph.rs` 102-119, 138-152, 174-184, 215-275,
  277-322, 336-369, 390-415, 436-461, 530-569; directory paths at
  `graph.rs:241, 272, 344, 366`. Four sorted `assert_eq` lists are
  re-sorted (`tests.rs:280-295`, `graph.rs:253, 270, 357-364`).
- Two checks would pass while covering less: `graph.rs:375, 379-380`
  (prefixes `jig-protocol-` and `jig-test-` drop the wire, the link and
  `jig-client-test-*`) and `graph.rs:394-397` (`jig-shell-` and
  `jig-protocol-` drop the native shell, the wire and the link). Each
  gets the `jig-client-` prefix too.
- Golden trees move with their crates:
  `crates/jig-protocol-client-wire/golden/v7` (3,032 files),
  `testing/jig-test-protocol-client-wire/golden/v1` (446), referenced by
  path at `tests/client-wire/tests/drift.rs:7`,
  `fuzzy_client_wire.rs:721`, `test_client_wire.rs:7`.
- Nothing in `scripts/`, `.config/nextest.toml` (binary names only),
  `clippy.toml`, `rustfmt.toml` or `tests/boundary/fixtures/names.txt`
  names them. Per-crate `clippy.toml` files move (`jig-shell-native`,
  `jig-test-client-records`).

## 3. Size and cut

97 renamed production lines in 42 files, about 3 rewrapped, 44 `use`
lines re-sorted: 110 to 150 edited lines, about 205 files, about 3,900
files moved (3,478 goldens). Three increments, in series (they share
the workspace `Cargo.toml`, `Cargo.lock`, `graph.rs`, `tests/client`,
`tests/ops`): the wire, the link, the shell and the test wire and
records (102 files); the view and the test kits (103 files); `ops` (25
files). The last such rename, `3bcb3db2`, was 558 files.

## 4. After step 1, against the rule

- `jig-domain-client` alone breaches it; step 4 splits it.
- `jig-protocol-transcript` is kept by README 15, though only client
  crates and `jig-ops-world` use it.
- Not breaches: `jig-ops-domain-connectors` (README 15),
  `jig-ops-world` (`examples.md`, 2), `jig-one-process-world` (both
  ends), `jig-test` (client crates as dev-dependencies only),
  `jig-test-connector` (shared vocabulary).

## 5. A mechanical definition, tried at the base

Over the normal and target dependencies `parse_manifest` reads:
- **seeds:** a crate named `<p>-client` or `<p>-client-*` but the wires
  and records (`-client-protocol-wire`, `-client-test-protocol-wire`,
  `-client-test-records`, which the engine reads too), and skein's
  `skein-ui*` and `skein-fake-person*`;
- **(a) downward:** a crate depending on a seed is client-side;
- **(b) upward:** a crate whose dependents, the allowed ones ignored,
  are all client-side is client-side (it catches `jig-domain-client`, a
  leaf);
- **(c) name shape:** after `<p>-`, a `client` segment, `shell-` or
  `view` makes a crate client-side, but `<p>-engine-*`;
- **allowed:** README 15's engine-process and shared crates
  (`jig-protocol-engine-clients`, `jig-protocol-engine-clients-http`,
  `jig-engine-clients-world`, `jig-fake-parties`,
  `jig-protocol-transcript`), `jig-test`, `jig-test-service`,
  `jig-one-process-world`, `jig-ops-world`; in temper, its binary
  `temper` and `temper-one-process-world`.

At the base, after step 1: in jig it flags exactly `jig-domain-client`;
without the allowed list, (a) also flags `jig-protocol-transcript`,
`jig-one-process-world`, `jig-ops-world`; without the wires' exclusion,
`jig-protocol-config`, `jig-protocol-engine-clients`, `jig-test-service`
and `jig-engine-clients-world`. In temper, (a) flags
`temper-shell-terminal`; (c) alone catches `temper-protocol-client-wire`
(its server depends on it); no rule flags
`temper-engine-domain-forge-client`, which is right (a substring test
for `-client` would).
