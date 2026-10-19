# Session 02: temper on `skein-forge-client`

Put temper on skein's bounded client and its measure, give effects' purposes
to temper's forge connector, and delete temper's copy of the client. One
increment: the purposes' move is about 90 new lines (their 88-line encoding
moves from the client), under the 300 that would split it into a 02.2, so
the repin, the purposes, the client's users and the deletion land together,
in four commits.

**Needs at start:** **skein 02.2** on skein's `origin/main` (README): the
client with `effect_key` (02.1) and the measure (02.2, decision 24); and
**temper 01.3**, the component on skein's call plans. Skein 02 needs temper 01.1. Skein 02.3 and 02.4 may or may not have landed; the repin
repairs what they break (02.1, Shape).

## Read first

- skein `docs/design/forge-client.md` 2 and 7, `forge.md` 2.3 and 3.7;
  temper `docs/design/domain/forge.md` 5 and 6 (the effects and their
  purposes).
- README (sessions, Rules, Done when), `code-organization.md` 1-3 and 5-8,
  `decisions.md` 2-5, 13 and 24, Design questions 14 and 17.
- `skein/02-client.md` (02.1's `effect_key`, 02.2's “temper's side”, 02.3's
  `bind_deployment`, 02.4's key fixtures); findings `survey-02-client.md`,
  `trace-client.md`, `base-runs.md`, `base-temper-02.md`.

## What the design asks

- forge-client 7.1: the client frames opaque identity bytes into a key
  (version 1, a two-byte namespace length, the namespace, the identity); it
  never inspects the identity. The owner supplies both.
- forge 2.3 and domain/forge 6: the purposes and their tags stay with the
  connector. No key at rest changes: the keys of tags 1 to 5 keep their bytes.
- forge 3.7: one fixed-width measure, the dialect's and the client's.

## Choices this plan makes

- Counts are **at the base** (temper dd12b8e). After temper 01.1 the files
  that name only the vocabulary name `skein_forge`; this session moves the
  lines that name the client's state. At the base 716 lines in 103 files
  name the client (193 by the crate's name, 523 through the files' alias);
  561 in 48 files reach its state and change: `findings/base-temper-02.md`.
- No alias: no `as` on the kit's crate, no `mod`, no re-export. Every
  `use` and path names `skein_forge_client`; rustfmt reflows the longer name.
- `EffectPurpose` is the connector's: `temper_engine_domain_forge::purpose`.
  Temper hands the kit no new value here: the namespace stays the
  deployment's 16 bytes (`bind_deployment`), the writers stay `Config`'s.

## Rules that bind it

- The repin is the first commit: `heavy cargo update -p skein-lib`, then
  `heavy cargo check --workspace --all-targets`. A Cargo.lock conflict takes
  `origin/main`'s and runs the update again, never a hand merge.
- No behaviour change, no raised bound, no loosened assertion. The sole
  allowance exception is the derived owned-example `engine.memory_bytes`
  correction below (README, Derived owned-example memory); operational
  admission caps and limits stay unchanged. Moved tests keep their names.
  New units under 20 ms, worlds under 0.5 s, unoptimized.

## Increments

### 02.1 The repin, the purposes, the client's users, the deletion

Scope: forge-client 7.1 and forge 2.3, 3.7; domain/forge 5 and 6.
Needs: skein 02.2 and temper 01.3 on `origin/main` (README); for its naming
  proof and its landing, jig01.s1's boundary check repair on jig's `origin/main`
  (README, Rules: the boundary check's jig repair). Skein 02.3's `bind_deployment`
  guard and 02.4's frame checks may be on it; if they are, the repin commit
  repairs the two places below.
Files: Cargo.toml (root: delete the member `crates/temper-engine-domain-forge-client`
  and its `path` entry, add `skein-forge-client = { git = "https://github.com/forgetic/skein.git" }`
  beside the other skein entries) and Cargo.lock; the 15 manifests naming
  `temper-engine-domain-forge-client.workspace = true` at the base:
  `crates/temper-{engine-domain,engine-domain-forge,engine-protocol-forge,engine-smith,protocol-store,service}`,
  `tests/{client,codecs,conformance,system,engine/domain,engine/forge,engine/forge/client,engine/protocol-forge}`,
  `tools/forgejo-conformance` (the line becomes `skein-forge-client.workspace = true`;
  `tests/client`, whose `front.rs` uses only `api`, drops it).
  Changed Rust: the 48 files with client state at the base (561 lines; by crate in findings).
  New: `crates/temper-engine-domain-forge/src/{purpose.rs,tests/purpose.rs}`,
  with `pub mod purpose;` in `lib.rs` and `mod purpose;` in `tests.rs`.
  Deleted: `crates/temper-engine-domain-forge-client/`.
Repin compatibility Files (README): `crates/temper-service/src/{worker_route.rs,
  tests/worker.rs}`, `crates/temper-worker-domain/src/tests/fixtures.rs`,
  `tests/worker/domain/src/protocol.rs`. Set the three Start literals to
  `live: None`; in worker_route::{process,channel}, explicitly ignore only
  `OpenEvent::Live` beside existing ignored variants; refresh codec-derived values.
Derived fixture Files: `docs/examples/owned.json`; existing regression in
  `crates/temper/tests/startup.rs::serve_prepares_once_on_the_owned_example`,
  whose body stays unchanged. Its consumers are `crates/temper/tests/support/owned.rs`
  (`EXAMPLE`), `tests/{startup,configuration,process}.rs` under that crate,
  and the copy command in `docs/guide/box.md`; preserve them and their assertions.
Shape: four commits, each builds and passes its crates' tests.
  1. **The repin**: update and check; repair there what skein's steps broke
     since the last repin (at the base the check fails on `Op::Lock` and jig's
     `Event::Locked`, which the tools plan's first repin, landed before this
     plan's first, repairs: findings).
     It also takes jig's boundary-check repair (README, Rules); a commit 1
     made before that repair landed is amended to take it.
     For this first compatible repin, also await jig01.s2 and take its
     landed compatible jig source, retaining a newer compatible revision.
     README's Settled smith adapters rule assigns this repin the three
     Start live:None literals and explicit ignored OpenEvent::Live arms;
     refresh the derived encode_start proof with the landed codec. This
     follows live-text's director decision and next-repin assignment.
     Refresh reached copies/derived values and name them in the commit
     body; keep all bounds, assertions and existing proofs, subject only to
     the derived owned-example allowance correction in commit 3 below.
     If skein 02.3 is on `origin/main`, `bind_deployment`
     answers false once a Restore, Keep, Make or Remake was taken: the one
     caller, `restart.rs`'s deployment row (via `domain.rs`'s wrapper), already
     calls `restart_refuse` on false; a test restoring a forge row before its
     deployment row would take a restart's order (at the base none:
     `rg -n bind_deployment` finds only `restart.rs:397` and the wrapper). If 02.4
     is on it, the key fixtures of `tests/engine/forge/client/tests/{simulation,
     fuzzy_simulation}.rs` (`deployment:goal`, `[entry]`, the world's namespace
     `world`) take the frame `\x01\x00\x05world` + identity, as 02.4 says.
     The plans now measure with skein 02.2's widths, so this commit also puts
     `crates/temper-engine-protocol-forge/src/tests/{reads.rs:489,contents.rs:259}`
     on `skein_forge::measure::answer` (the crate already depends on skein-forge)
     and changes the two `contents.rs` values (Test shape).
  2. **The purposes**, with the root `skein-forge-client` entry and the
     connector's manifest line it needs: `purpose.rs` (sketch, compiled with a
     stand-in for the kit's `effect_key`; clippy clean). The tag, then each field big-endian,
     is the identity; the kit frames it. Sizes: 26, 25, 11, 18, 11 + resource.
     ```rust
     // crates/temper-engine-domain-forge/src/purpose.rs
     #[derive(Clone, PartialEq, Eq, Hash, Debug)]
     pub enum EffectPurpose {
         ProjectionHistory { goal: u64, task: u64, position: u64, family: u8 }, // tag 5
         Call { task: u64, attempt: u64, completion: u32, position: u32 },      // tag 1
         Step { task: u64, purpose: u16 },                                      // tag 2
         Projection { goal: u64, part: u8, number: u64 },                       // tag 3
         Release { task: u64, resource: Box<[u8]> },                            // tag 4
     }
     impl EffectPurpose {
         /// Tag, then fields big-endian; `ProjectionHistory`: tag, family, goal, task, position.
         #[must_use] pub fn identity(&self) -> Option<Box<[u8]>>;
         /// `skein_forge_client::effect_key(namespace, &self.identity()?, max_bytes)`.
         #[must_use] pub fn key(&self, namespace: &[u8], max_bytes: u32) -> Option<Box<[u8]>>;
     }
     ```
     Built with `skein_lib::List` (no `Vec`: the lints ban it). Callers:
     `domain.rs`'s `projection_key`, three calls, become
     `EffectPurpose::ProjectionHistory { goal, task, position, family: 1 }.key(namespace, l.client.op_bytes)`
     (family 2 with `position: revision`) and
     `EffectPurpose::Projection { goal, part, number }.key(..)`; `forge_route.rs`'s
     `effect_key(domain, key)` builds `Call { task, attempt, completion, position }`
     and `.key(&domain.core.counters.deployment().id, 44)`, `expect("fixed call key fits its frame")`.
  3. **The users**, in the other 14 manifests and the 48 files: every `temper_engine_domain_forge_client` and
     `client::`/`forge_client::`/`d::` path to client state becomes `skein_forge_client::`;
     vocabulary stays `skein_forge`. Every alias of the crate goes (`as client`,
     `as forge_client`, `as d`, `{self as client, ..}`: `rg -n
     'temper_engine_domain_forge_client( as |::\{self as)' crates tests tools` finds 43 at the base). The nine measure callers change too:
     `effect_bytes` (domain-forge `lib.rs`, three) -> `skein_forge_client::measure::effect`;
     `stored_bytes` (`lib.rs`) -> `skein_forge_client::measure::stored`;
     `answer_bytes_unbounded` (`lib.rs`; `temper-engine-domain`'s `store.rs`
     `call_answer_bytes`) -> `skein_forge::measure::answer`; `domain.rs`'s
     `keep_named_answer`: `skein_forge_client::measure::answer_rows(value,
     limits.client.rows)` and `skein_forge::measure::answer(value)` compared with
     `limits.client.answer_bytes`: admitted only if `answer_rows` is true and the
     measure is `Some(b)` with `b <= answer_bytes`, else `Err(answer)` as at the base
     (an `Err` or a non-`Read` payload stays admitted). If an earlier repin already
     repaired these (`rg` for the old names finds none), only the users move.
     The complete users cutover also refreshes the owned example's derived
     `engine.memory_bytes` from `system::startup_worst_case`, with all inputs
     and operational limits unchanged (README, Derived owned-example memory).
     Skein02.3's `Domain::identity_used` grows the inline client from 592 to 600 bytes,
     carried once by `system::memory`'s `size_of::<Service>()`: complete sum
     78,675,187,547,016 becomes 78,675,187,547,024 on the checked tree. Record
     both complete sums and this cause; after a rebase rederive the whole sum,
     rather than applying a stale delta. Refresh any further reached copy or
     derived value and list every user in the commit body.
  4. **The deletion**: the crate's directory, its member and its path entry.
Test shape: **2 tests move, 74 leave with the crate.** `cargo nextest list -p
  temper-engine-domain-forge-client` shows 76 at the base (0.084 s); the two
  purpose tests keep their names and move to `tests/purpose.rs`, adapted to
  `purpose.key(&[1; 16], N)`: `effect_keys_are_stable_by_purpose_and_separate_deployments`
  (44-byte cap, 43 refuses) and `projection_history_keys_keep_task_position_and_history_family`
  (45 bytes, 44 refuses). Skein 02.1 holds the other 74. Three new units, <20 ms:
  `identity_bytes_pin_tags_one_to_five` (one row per tag: `Call{7,2,3,4}` ->
  `[1, 0,0,0,0,0,0,0,7, 0,0,0,0,0,0,0,2, 0,0,0,3, 0,0,0,4]`; `Step{7,9}` ->
  `[2, 0*7,7, 0,9]` (`0*7` is seven zero bytes); `Projection{1,2,3}` -> `[3, 0*7,1, 2, 0*7,3]`; `Release{7,"ab"}`
  -> `[4, 0*7,7, 0,2, b'a', b'b']`; `ProjectionHistory{1,2,3,family 1}` -> `[5, 1,
  0*7,1, 0*7,2, 0*7,3]`), `a_call_key_is_the_framed_forty_four_bytes` and
  `a_history_key_is_the_framed_forty_five_bytes`, each asserting the whole key
  against a literal and `None` one byte under the cap. Namespace `[7; 16]`:
  ```rust
  #[rustfmt::skip]
  const CALL_KEY: [u8; 44] = [ 1, 0, 16, 7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
      1, 0,0,0,0,0,0,0,7, 0,0,0,0,0,0,0,2, 0,0,0,3, 0,0,0,4 ];
  #[rustfmt::skip]
  const HISTORY_KEY: [u8; 45] = [ 1, 0, 16, 7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
      5, 1, 0,0,0,0,0,0,0,1, 0,0,0,0,0,0,0,2, 0,0,0,0,0,0,0,3 ];
  ```
  The vectors are the base client's bytes (checked in the scratch against
  the base `effect_key`, all five tags). No other test changes name or value but the three below.
  skein 02.2 puts the plans' `reads::size::answer` on the measure's widths
  (decision 24), and its "temper's side" names the three component tests
  that follow: `the_running_answer_size_agrees_with_the_clients_measure`
  (`tests/reads.rs`) passes unchanged; `contents_sizes_agree_with_the_clients_measure`
  (`tests/contents.rs`) expects pull files 68 and comparison 100 (52, 84), and
  `raw_sides_at_the_answer_bound_fit_and_one_byte_more_is_too_large` sides of
  4031 and 4032 (4047, 4048). Those are the only component-test values that change. If a side
  disagrees otherwise, stop and return the question; never loosen an
  assertion, drop a row or raise a bound.
Proof: unit; `heavy cargo test --locked --offline -p temper --test startup
  serve_prepares_once_on_the_owned_example -- --exact --nocapture`: the same
  existing regression passes with only the derived allowance correction,
  no new test or assertion change. Complete the required reverse dependents
  and all remaining joint/process/gate proofs; the scratch diagnostic is not
  a substitute for them.
Proof: unit; `heavy cargo nextest run -p temper-engine-domain-forge tests::purpose`:
  5 passed, each <20 ms (0.002 to 0.003 s in the scratch; the two moved
  ones took 0.004 s and 0.006 s at the base).
Proof: unit; `heavy cargo nextest run -p temper-engine-domain-forge`: origin/main's
  count at the step's start plus 5 (27 from the base's 22, 0.015 s), <1 s.
Proof: joint; `heavy cargo nextest run -p temper-engine-domain-forge
  -p temper-engine-domain -p temper-engine-smith -p temper-protocol-store
  -p temper-service -p temper-engine-forge-client-world -p temper-engine-forge-world
  -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world
  -p temper-engine-domain-world -p temper-conformance-world -p temper-codecs-world
  -p temper-client-world -p temper-system-world -p forgejo-conformance`: the
  set's count at the step's start plus 5 (926 from the base's 921, 4.823 s; the
  client's 76 are not in it): the five purpose tests are the only change; <15 s. A failure of a profile, limits or
  size test after the measure is a design question: return it, change no bound,
  except for the fully traced derived owned-example allowance correction
  explicitly authorized above. An unexplained difference still returns.
Proof: joint; the same command with `--profile fuzzy`: its count at the step's
  start, unchanged (39 at the base, 20.410 s, its slowest `fuzzy_walking` 20.4 s); <60 s.
Proof: process; `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  heavy --gate scripts/proof-one-process.sh` (README, Rules): its count at the step's
  start passed (19 at the base), <5 minutes (145.694 s in `base-runs.md`, 149.083 s alone here). Once: the keys
  at rest and the engine's calls go through the new client.
Proof: joint; `heavy --gate scripts/check.sh`: origin/main's focused count at the
  step's start less 71 (the client's 76 out, 5 in: 1808 from the base's 1879) and
  its fuzzy count unchanged (50 at the base; 3.754 s and 16.803 s), clean
  clippy and format, focused <15 s, fuzzy <60 s.
Don't: add an alias, a re-export or a `mod client`; change
  `tests/client/tests/names.rs` or exempt a crate there; change a tag, a field's
  order or width, or any key at rest; change an admission bound, a limit or a
  test's value to pass, beyond skein 02.2's three and the derived owned-example
  allowance correction above; write `temper` into skein; move `EffectPurpose` into
  skein or give the kit a purpose; touch jig's own, different
  `jig_domain_core::EffectPurpose` (`forge_route.rs` names it as an effect's
  `origin`).
Size: about 680 lines moved (changed callers counted as moved: 561 lines
  of client state in 48 files, the callers among them; 88 of the purposes' encoding, the client's
  `identity.rs`; 31 of the two moved tests), 90 new (the three new tests and
  their vectors about 72, `key` and the module's lines about 15, the manifest
  and root entries about 3). Under 300: no 02.2.
Removes: `crates/temper-engine-domain-forge-client/`, 7,209 lines at the base
  (4,913 source, 2,283 test, 13 manifest; about 6,757 after 01.1 took the
  vocabulary and `locate.rs`): its 74 other tests live in skein; its
  `EffectPurpose` and `effect_key` live on in `purpose.rs`.

## Done when

- `heavy cargo check --workspace --all-targets` prints `Finished`, exits 0.
  The gate (`heavy --gate scripts/check.sh`) is land's.
- `heavy rg -n 'temper_engine_domain_forge_client|temper-engine-domain-forge-client' Cargo.toml Cargo.lock crates testing tests tools -g '*.rs' -g Cargo.toml -g Cargo.lock`
  prints nothing (exit 1); at the base it prints the crate and its users.
- `heavy rg -n 'skein_forge_client( as |::\{.* as )' crates testing tests tools -g '*.rs'`
  prints nothing (exit 1): no alias of the kit's crate.
- `heavy rg -n 'answer_bytes_unbounded|effect_bytes|client::(answer_bytes|stored_bytes)\b' crates testing tests tools -g '*.rs'`
  prints nothing (exit 1): the old counters are gone.
- `heavy rg -l 'enum EffectPurpose' crates testing tests tools -g '*.rs'` prints
  one path, under `crates/temper-engine-domain-forge/src/`, and nothing else.
- `heavy python3 -c 'from pathlib import Path; print("client_copy_absent=" + str(not Path("crates/temper-engine-domain-forge-client").exists()))'`
  prints `client_copy_absent=True`. At the base it prints False.
- `heavy cargo nextest run -p temper-engine-domain-forge tests::purpose`
  shows 5 passed.
