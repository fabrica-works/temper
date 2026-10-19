# Survey 02: the client (`temper-engine-domain-forge-client`)

Read only, temper at origin/main. Designs: skein `forge-client.md` (FC), `forge.md` (F), temper `domain/forge.md` (TF).
Paths are under `temper/crates/temper-engine-domain-forge-client` unless stated. The graph was used once (see d); the rest is `rg`/`sed`/`wc`.

```
$ wc -l Cargo.toml src/*.rs src/tests/*.rs ; grep -c '#\[test\]' src/tests.rs src/tests/*.rs ; grep -n 'cfg(test)' src/*.rs
$ grep -n 'jig\|smith\|temper' src/*.rs Cargo.toml ; grep -h skein_lib src/*.rs | sort | uniq -c ; rg -c serde .
```

## (a) Modules and dependencies

| file | prod | test | #[test] | holds |
|---|---|---|---|---|
| lib.rs | 52 | 0 | 0 | `no_std`, re-exports, 4 `*_bytes` wrappers |
| api.rs | 850 | 0 | 0 | 1-417 vocabulary (29 types); 419-850 `Shape`, `accepts`, 8 helpers |
| boundary.rs | 295 | 0 | 0 | `Event`, `Request`, `Entry`, `Outcome`, `Recovery`, `Stored`, `Watch`, `What` |
| bounds.rs | 285 | 0 | 0 | admission `op`, answer size, deep byte counts |
| calls.rs | 384 | 4 | 0 | budget: four classes, reserve, window, rate refusal, retries |
| domain.rs | 235 | 0 | 0 | `Domain`, `step`, `fire`, `resume`, `bind_deployment` |
| facts.rs | 32 | 0 | 0 | `Fact`, `Priority` |
| identity.rs | 195 | 0 | 0 | `EffectPurpose`, `effect_key`, `Config`, `Writer`, `valid`, `historical` |
| keep.rs | 1377 | 0 | 0 | working set, listing, drift, echoes, adoption |
| limits.rs | 90 | 0 | 0 | `Limits` (24 fields), `worst_case` |
| locate.rs | 35 | 0 | 0 | locate's bytes; nonzero id |
| outbox.rs | 1079 | 0 | 0 | lanes, recovery, finding again, restart, settle |
| tests.rs, tests/{keep,outbox,locate}.rs | 0 | 749+784+719+31 | 31+19+24+2 | in-crate step tests |
| total | 4909 | 2287 | 76 | with Cargo.toml (13): 7209 |

calls.rs:80 is `#[cfg(test)] pub(crate) const fn remaining` (the 4 test lines).
- Cargo: `skein-lib` only (`Deadlines Duration Env Id List Map Queue Rng Slab Time Token Wall`). No temper, jig, smith,
  skein-json or external crate.
- **Blockers: none.** jig appears in 5 doc-comment lines only (lib.rs:2, boundary.rs:1 and 30, identity.rs:11, outbox.rs:2). The
  crate will need `skein-forge` for the vocabulary.
- (e) serde and serde_json: 0 in every file (no dependency).

## (b) Map to the design

| part | goes to | design |
|---|---|---|
| api.rs 1-417: `Commit Repository Hint Change Op Read Write Answer Error Kind State Ci Check Verdict Permission BranchCreation Summary Provenance Comment Pull Review Remark Status JobAttempt Step File Protection Settings Collaborator` | skein-forge `api` | F 3.1-3.6 |
| locate.rs | skein-forge `locate` | F 2.1, 3.7 |
| api.rs 419-850 `accepts` (432) | placement open: skein-forge or client | F 3.7 table, FC 2.3 |
| bounds, calls, domain, facts, limits | skein-forge-client | FC 2.1-2.3, 6 |
| keep (1377) | skein-forge-client | FC 3, 4, 5, 7.6 |
| outbox (1079), boundary (295) | skein-forge-client | FC 7.2-7.5, 8 |
| identity: frame, `Config`, `Writer`, `valid`, `historical` | skein-forge-client | FC 7.1 |
| identity: `EffectPurpose` and its tags | temper's top (stays) | F 2.3; TF 5, 6 |

## (c) Temper-specific today (the design makes them the owner's)

- identity.rs: `EffectPurpose` (`Call` tag 1, `Step` 2, `Projection` 3, `Release` 4, `ProjectionHistory` 5) and
  `effect_key(deployment, &EffectPurpose, max)`, which encodes the identity itself (sizes 25, 11, 18, 11+len, 26). FC 7.1:
  `effect_key(namespace, identity, max)` over nonempty opaque bytes. An API change.
- domain.rs:52 `bind_deployment(id: [u8; 16], limits)`: FC 2 keeps it, so no gap.
- Identity builders: domain-forge/src/domain.rs:2677, 2684, 2697; engine-domain/src/route/forge_route.rs:328 (`Call`, `op_bytes` hard-coded 44).
- tests.rs:39 and :672 test `EffectPurpose`. Namespace, writers, `Limits` values and `Watch` are already passed in.

## (d) Audit against forge-client.md (V verified; N not verified or differs)

- V settle: limits.rs:71 `settle == ZERO` is the only use; no window, no reread; tests.rs:748 asserts `worst_case` none for 0.
  The reread is temper's: domain-forge/src/settle.rs:94 reads `env.limits.client.settle`.
- V no start reads: the crate emits only `Request::Call` for ops the owner gave. Identity and page-cap reads are the
  component's and dialect's (F 4.1, forge-connection.md 3.3).
- V admission (FC 2.3): bounds.rs:26 (op_bytes, `max_bytes` 1..`answer_bytes`, page >= 1, path nonempty); `Busy` from domain.rs:170.
- V answer check: calls.rs:279-290 (`TooLarge` read, `InvalidAnswer` write, else `accepts`).
- V limits (FC 2.2): limits.rs:49-75 matches, including which may be 0. V `Config` (FC 7.1): identity.rs:112; no config means
  `Forbidden` and no key believed.
- V recovery rules of FC 7.3-7.5: tests/outbox.rs, 24 tests (start and attempt saved first, uncertain find, raced, deadline held,
  original deadline across restart and changed clock, copied/revised/unknown rows never settle).
- V every table full against `worst_case`: exists, but in the client world, not in-crate: tests/engine/forge/client/tests/memory.rs:232.
- N Busy "never a dialect's" (FC 2.3): a dialect's `Err(Busy)` passes calls.rs:330 unchecked.
- N golden bytes for keys (FC 10): tests.rs:39 and :672 assert stability, inequality and length 45, not bytes.
- N echo landing (FC 7.6 "brings the next read forward to `hinted`"): keep.rs:1301-1331 `made` sets `row.hinted` only; no `Env`, no
  alarm re-armed (hints do arm it, keep.rs:667, 707).
- N panics on owner misuse: `assert!`/`expect` for entry numbers and duplicate rows (outbox.rs:208, calls.rs:165, 252, 257);
  FC names none. N `Restore` before a namespace is not refused (an owner duty in FC 2, 8). N no provenance on an issue find.
- Graph: `trace_path` on `identity.effect_key` showed only the 2 in-crate tests as callers; the outside callers use
  `use ... as client`, which the graph misses. `rg` is the authority for users.

## (f) Users

```
$ rg -n 'temper-engine-domain-forge-client' --glob Cargo.toml .
$ rg -c 'temper_engine_domain_forge_client' --glob '*.rs' .   # summed per crate
$ rg -o 'client::[A-Za-z_]+' crates/temper-engine-domain-forge/src | sort | uniq -c
```

15 Cargo dependents; 193 lines name the crate in `.rs`:

| crate | lines | note |
|---|---|---|
| crates/temper-engine-protocol-forge | 32 | the component today (`api`) |
| crates/temper-engine-smith | 22 | `Outcome`, `api` |
| crates/temper-protocol-store | 21 | durable rows (`records/client.rs`, `keys.rs`) |
| crates/temper-engine-domain-forge | 21 | the top (8333 lines): `Event 50 Outcome 66 What 41 Request 24 Made 22`, 387 `client::api` paths |
| crates/temper-engine-domain | 20 | `store.rs` 10, `forge_route.rs` |
| tools/forgejo-conformance 13, tests/engine/protocol-forge 13 | 26 | `api` |
| tests/engine/domain 10, tests/conformance 10, crates/temper-service 10 | 30 | `Limits`, `api` |
| tests/engine/forge/client 8, tests/engine/forge 6, tests/system 5, tests/codecs 1, tests/client 1 | 21 | worlds |

`-forge-change` (1330 lines) and `-forge-issues` (656 lines) do not depend on it (Cargo: `skein-lib` only; 0 uses).

## (g) Tests

| where | size | #[test] | exercises |
|---|---|---|---|
| in-crate `src/tests*` | 2283+4 | 76 | client only, moves; but tests.rs:39 and :672 are `EffectPurpose` |
| tests/engine/forge/client (`temper-engine-forge-client-world`) | src 1430, tests 1347 | 31 | client only, over `temper-fake-forge-domain` (skein-lib only); moves (FC 10) |
| tests/engine/forge (`temper-engine-forge-world`) | src 796, tests 2008 | 55 | temper's top with `-issues`, `-change`; stays |
| tests/engine/protocol-forge | 13 use sites | not counted | component with client (FC 10 protocol world) |
| tests/engine/domain, conformance, system, codecs, client; protocol-store | | | temper's routes and stores; stay |

The client world's `translate.rs` (567 lines) maps client `api` to the fake's types; the move waits on or ships with the fake forge domain.

## Gaps for the plan

- `EffectPurpose` to the top; `effect_key` to FC 7.1's opaque identity, with golden vectors.
- Decide where `accepts` lives; reword 5 jig doc lines; review panics; add a Busy-from-dialect rule; re-arm on echo or amend FC 7.6.
- Retarget 15 Cargo files and 193 lines; `op_bytes` 44 in forge_route.rs:328 should come from `Limits`.
