# Design trace: the client (skein forge-client.md 2-10; temper domain/forge.md 5-6)

Read only. Code: `temper/crates/temper-engine-domain-forge-client/src` (C), world `temper/tests/engine/forge/client` (W).
Base drift: outline says skein b4cc6eb, temper 62d50e0b8. Now both repos' origin/main are skein d120820, temper 27995fb75.
`forge-client.md` is absent at b4cc6eb (added 427 lines by d120820); temper `domain/forge.md` changed (+142 -100). C and W are
unchanged between the two temper commits. The outline's base line should name d120820 and 27995fb75.

```
$ git -C skein rev-parse --short HEAD ; git -C skein rev-parse --short origin/main ; git -C temper rev-parse --short origin/main
$ git -C skein diff --stat b4cc6eb d120820 -- docs/design/forge-client.md ; git -C temper diff --stat 62d50e0b8 27995fb75 -- crates/temper-engine-domain-forge-client tests/engine/forge/client
$ sed -n 233,411p temper/docs/design/domain/forge.md ; sed -n 330,360p skein/docs/design/forge.md
$ for f in tests.rs tests/keep.rs tests/outbox.rs tests/locate.rs; do grep -A1 '#\[test\]' $f | grep 'fn '; done   # in C/src
$ grep -A1 '#\[test\]' tests/simulation.rs | grep 'fn '                                                          # in W
$ rg -n 'max_out|observed: None|participat|Error::Refused|Fact::|Withdraw|Remake|Held' C/src/tests.rs C/src/tests
$ rg -n 'Pause|ReadAfresh|SettleOutbox|OutboxDone|Remake' W/src W/tests ; rg -n 'restoration_failed|bind_deployment' temper/crates tests
$ rg -n 'page: 0|max_bytes: 0|path: Box::new' C/src/tests.rs C/src/tests W ; sed -n 20,70p C/src/bounds.rs ; grep -n size_of C/src/bounds.rs
$ rg -n '\bBusy\b' temper/crates/temper-engine-protocol-forge temper/crates/temper-forge-forgejo -g '*.rs' ; sed -n 320,335p C/src/calls.rs
$ sed -n 224,241p C/src/outbox.rs ; sed -n 52,62p C/src/domain.rs ; sed -n 100,125p C/src/keep.rs ; sed -n 414,434p C/src/keep.rs
$ rg -n 'pub struct (Summary|Comment|Review)' -A12 C/src/api.rs ; rg -n 'locate::(bytes|accepts)' C/src
$ rg -n 'client::(stored_bytes|effect_bytes|answer_bytes)' temper/crates -g '*.rs' ; rg -n 'forge_client' temper/crates/temper-engine-protocol-forge/src
$ sed -n '/fn effect_keys_are_stable/,/^}/p;/fn limits_reject_incoherent/,/^}/p;/fn deployment_config_and_decoded/,/^}/p' C/src/tests.rs
```

## 1. Gaps of kind (b): the design is silent or ambiguous on something observable

1. **The measure of bytes.** 2.3 admits an op when "owned bytes" <= `op_bytes` and checks answers against `answer_bytes`/`rows`;
   it defines neither. C counts byte lengths plus `size_of::<Summary|Comment|Review|Status|Remark|File|Collaborator|Commit|Box<[u8]>>()`
   per row (bounds.rs:86-250), so a limit's effect depends on rustc's layout. Public `stored_bytes`, `effect_bytes`, `answer_bytes`
   (lib.rs:35-52) are used by temper's top (domain-forge/src/lib.rs:150, 370-373; domain.rs:178) and are not in the design.
   Options: (1) fix the widths in 2.3 (a table: lengths + 8 per number, 32 per hash, ...) and name the three functions; (2) say "the
   client's own `size_of` accounting" and test only at the boundary; (3) measure the encoded form. Recommend 1 (a persisted limit
   must not move with the compiler); it is a behaviour change, so S02.2.
2. **Order of two refusals.** Keep (3), Make (7.2) and Read (5) list causes, not which wins. C: Keep TooLarge(count) > Busy(state) >
   per-watch TooLarge/Refused > Busy(capacity); Make Busy(not ready) > Forbidden > TooLarge > Busy(entries full), then `assert!`s;
   Read TooLarge > Busy. Options: (1) state C's order and test one pair each; (2) "any one", test singles. Recommend 1.
3. **Owner misuse is asserted but unlisted.** C panics on: a Make number not above every held one or a duplicate (outbox.rs:208),
   a duplicate Restore row, Restore after Restored (keep.rs:415), an Answered for an unknown token (calls.rs:165, 252, 257), limits
   without `worst_case`. 7.2 lists "a new entry's number is above every held one" beside refusals. Options: (1) a Preconditions list in
   2, "asserted, never answered" (programming-model); (2) answer each with `Failed(Refused)`. Recommend 1.
4. **Restore before a namespace** (outline S02.2 plans it "refused"). 2 and 8.1 make the namespace the owner's duty; C's
   `bind_deployment` (domain.rs:52) checks only config and validity, not that a Restore came first. No config is legal (every effect
   Forbidden), so "namespace present" cannot be the check. Options: (1) `bind_deployment` returns false, changing nothing, once a
   Restore or Restored was seen (add to 2); (2) assert; (3) drop it from S02.2. Recommend 1.
5. **A rate-refused write is sent again.** 6 requeues it under a new token spending no retry; 7.4.3 emits `Progress` with an attempt in
   the decision that sends. C emits a new `Attempt` (new deadline and `expires`) at every send, so the saved deadline moves. 7.5 says a
   find does not restart the saved deadline. No test: the only RateLimited on an owned call is tests/keep.rs:443, a read. Options:
   (1) each send is an attempt (C; say so); (2) a requeue keeps its attempt and emits no Progress. Recommend 1 (restart then
   looks for it, harmless), plus a test.
6. **An issue find has no provenance.** 7.3: a row settles a find only if "original (not revised, provenance known)" and keyed.
   `api::Summary` (api.rs:290) has `key`, `author`, `updated` only; C's issue find (outbox.rs `listed`) uses writer + key. Options:
   (1) 7.3 says the original rule is for comments and reviews, issues settle by writer + key; (2) add provenance and revision to
   `Summary` (forge.md 3, the dialect). Recommend 1.
7. **"`Busy` is never a dialect's"** (2.3). C passes a dialect's `Err(Busy)` through (calls.rs:330); no plan returns it today (rg).
   Options: (1) `accepts` maps an answered Busy to `InvalidAnswer`; (2) a dialect test in skein-forge only; (3) drop the sentence.
   Recommend 1.
8. **Withdraw of an entry with an attempt, or unknown** (7.2). C returns without an outcome or fact (outbox.rs:228). Options: (1) silent
   no-op, the entry reaches its own terminal (say so); (2) a fact; (3) assert. Recommend 1; add a test.
9. **After a failed restore** (8.1: "meanwhile `Keep` is `Busy`"). Nothing clears `failed` (keep.rs `is_ready`); `restoration_failed()` has
   no caller in temper; writes still run. Options: (1) failed is terminal, the owner reconciles its store and builds a new `Domain`
   (say so); (2) a `Reconciled` event; (3) the client drops bad rows. Recommend 1. Also 7.4.5 "`Exists` before an attempt fails
   `Exists`" is unreachable (`sent` always sets the attempt): delete the clause.

## 2. Gaps of kind (a), by increment (design clear; the outline adds them)

- **S01.2:** `locate::bytes` and `locate::accepts` are `pub(crate)` (used by bounds.rs:46 and api.rs:515); they are `pub` in
  `skein_forge::locate`. The deep-count `answer_bytes_unbounded` (bounds.rs:205) goes with the vocabulary (see section 5, 2).
- **S02.1:** a golden-bytes test of `effect_key` (10; N2): version 1, two-byte big-endian length, namespace, identity; none for an empty
  namespace, 65536 bytes, an empty identity, total over `max`. An opaque-identity stability test replaces tests.rs:39 and :672. A
  `bind_deployment` test (true replaces the namespace; false changes nothing without config, or when `op_bytes` is exceeded).
  The 5 jig doc lines and lib.rs's "domain/forge.md" cite are reworded.
- **S02.2** (tests, no design question): (i) restart events: `PauseOutbox`, `ReadAfresh` (each repository's listing from its stored mark,
  inclusive, and each live resource, as Fresh; `ReadAfreshDone` once), `SettleOutbox` (`OutboxDone` after each entry is looked for
  once), `Remake` lifting `Held`: no C test names any of them. (ii) `restoration_failed()` once per cause 8.1 lists, then `Keep` Busy.
  (iii) Keep: `Refused` for participation on a repository or branch, `TooLarge` for an empty or over-`op_bytes` branch name, `Busy`
  before restore and after a failed one (only the duplicate Refused, tests/keep.rs:198, and the batch cases exist). (iv) A hint with a
  branch name or key over `op_bytes` gives `Fact::Refused` and no read. (v) 2.2 as a table: each limit at 0 gives none except
  `op_bytes`, `answer_bytes`, `reserve`, `backoff_max`, `poll_max`, `facts`; `backoff > backoff_max`, `poll > poll_max` (the one test
  covers rate, calls > pending, reserve > rate, window, settle); `max_out` = max(4, 2r + 2p + 3), both arms. (vi) 2.3: page 0, empty
  file path, `max_bytes` 0 and over `answer_bytes` for each of Job and File: TooLarge with no call (only the +1 cases exist). (vii) a rate
  refusal on a write (gap 5). (viii) the echo landing brings the item's read forward (7.6; N3: `made` sets `hinted`, keep.rs:1331, no
  alarm). (ix) keep.rs (1377) and outbox.rs (1079) split under 800.
- **T02.1:** `EffectPurpose` and its tests land in temper's connector with golden vectors (forge.md 6; none today): tag 1 call 25 bytes,
  2 step 11, 3 projection 18, 4 release 11 + resource (length two bytes), 5 history 26; a key = 1 + 2 + 16 + those (call 44, history
  45); parts 1-9, family 1/2; numbers big-endian; the same thing asked again gives the same bytes. `op_bytes` 44 hard-coded at
  engine-domain/src/route/forge_route.rs:328 (a Call key only) comes from the key's own size, not a literal. temper's restart code
  (restart.rs:336-340) sends Pause/ReadAfresh/Settle; it never reads `restoration_failed()`.
- **S06.2:** W's `restart` (src/world.rs) sends Restore, Restored, Make only: add `Writer`, `PauseOutbox`, `ReadAfresh`,
  `SettleOutbox` and a `Held` then `Remake` story (10 names the restart and held stories; `Remake` appears in no test). The other
  stories of 10 exist in tests/simulation.rs (see 4). W's memory test stays on `skein_world::domain::heap::Counting`.

## 3. Survey findings "not verified" (survey-02 d): all true

- N1 true: calls.rs:330 lists `Error::Busy` among the passed-through errors; no `Busy` in temper-forge-forgejo or protocol-forge.
- N2 true: tests.rs:39 and :672 compare keys for equality, inequality, length 45 and the 43/44 refusals; no byte literal of a key.
- N3 true: keep.rs:1331 `row.hinted = true` in `made`; no `Env`, no alarm (hints arm it at keep.rs:667, 707).
- N4 true: `assert!`/`expect` at outbox.rs:208, calls.rs:165, 252, 257, keep.rs:415; design names none (gap 3).
- N5 true: `bind_deployment` and `Restore` check no order (domain.rs:52; keep.rs restore_live) (gap 4).
- N6 true: issue find uses writer + key (gap 6); `Summary` has no provenance.
- Also true: 76 in-crate tests, 2 are EffectPurpose's, so 74 move (outline). Client crate unchanged between the two temper commits.

## 4. Names the design gives (path: names)

- crates/skein-forge-client (no_std + alloc, lib-only; deps skein-lib, skein-forge vocabulary): `Domain::{new, seeded, configured,
  bind_deployment, next_deadline, is_due, is_ready, has_room, cached, pop_fact, restoration_failed}`, `step`, `fire`, `resume`,
  `max_out`, `Limits`, `worst_case`, `effect_key`, `Config{namespace, writers}`, `Writer{forge, author}`, `recovery(&Write)`,
  `Recovery::{Keyed, Conditional, Idempotent, Unrecoverable}`, `Priority::{Fresh, Write, Keep, Slow}`, `Fact`.
- Event: `Keep{owner, watches}`, `Hint`, `Writer{resource, taken}`, `Pushed`, `Make{entry}`, `Remake`, `Withdraw{entry}`,
  `Restore{record}`, `Restored{clock}`, `PauseOutbox`, `ReadAfresh`, `SettleOutbox`, `Read{owner, repository, read}`,
  `Answered{call, cost, result}`. Request: `Call`, `Read`, `Kept`, `Save`, `Erase`, `Progress`, `Outcome`, `Drift{resource, expected,
  observed}`, `Changed{resource, result}`, `ReadAfreshDone`, `OutboxDone`. Rows: `Stored::{Live, Repository}`, `Key`, `Entry`,
  `Position`, `Attempt{deadline, expires}`, `Watch`, `Cached`, `RecoveryClock::{Monotonic, Wall}`.
- Outcome: `Made{made, found}`, `Uncertain`, `Held`, `Raced{made, why}`, `Failed`, `Withdrawn`; `Made::{Commented, Reviewed}`;
  `Raced{Merged, Stale}`; `Condition`, `Effect`.
- Limits: pending calls op_bytes answer_bytes rate reserve window read_attempts backoff backoff_max resources repositories rows inbox
  poll poll_max hinted slow settle entries write_attempts lifetime clock_margin facts (24).
- In C, not in the design: `reclaim`, `facts_lost`, `stored_bytes`, `effect_bytes`, `answer_bytes`, `answer_bytes_unbounded`.
- skein-forge: `api` (vocabulary: 29 types), `locate`; `accepts` (forge.md 3.7; api.rs:473) goes with the client (S02.1).
- Key frame: byte 1 = version 1, two bytes big-endian namespace length, namespace, identity; namespace 1..=65535.
- Tests (10): in-crate step tests (budget order, reserve, refunds, rate refusal; listing ties and overlap; drift; start and attempt saved
  first; original deadlines; copied/revised/unknown rows; golden key bytes; every table full vs `worst_case`); world `tests/forge/client`
  (client alone, scripted durable parent, fake forge domain, seeded faults/restarts/rate resets) with a referee that reads no client
  state; `tests/forge/protocol`. W stories now: `creations_and_comments_survive_faults_late_landings_and_restart`,
  `an_uncertain_branch_is_found_without_retry_after_restart`, `a_deleted_uncertain_branch_is_held_after_its_deadline`,
  `a_conditional_merge_lands_once_within_its_attempt_lifetime`, `a_late_set_reaches_its_final_state`,
  `another_writer_taking_a_branch_name_is_reported_without_overwriting_it`, `every_live_fresh_read_finishes_once_under_faults_and_rate_resets`,
  `a_seed_replays_and_facts_change_no_decision`, memory.rs `full_live_caches_inbox_stamps_echoes_outbox_and_every_entry_point_fit`.
- temper (forge.md 5-6): limits `forge.client.{resources, repositories, poll_ms, poll_max_ms, hinted_ms, slow_ms, settle_ms, rate,
  window_ms, reserve}`; effect kinds 3-15 (3 open PR, 4 merge, 5 review, 6 status, 7 comment, 8 issue, 9 create branch, 10 edit PR,
  11 update, 12 close/reopen PR, 13 edit issue, 14 reviewers, 15 delete branch); identity tags 1-5, parts 1-9, family 1/2.

## 5. Contradictions with the outline

1. Design 10 puts "every table full at once against `worst_case`" among the in-crate step tests; the only test is W's memory.rs:232
   and S06.2 moves it to the world (skein's convention: world crates use `Counting`; C is `no_std`, skein-lib only). Amend 10 to
   name the world (recommended), or add an in-crate test.
2. S01.3 moves protocol-forge's plan tests, which call `client::answer_bytes_unbounded` (tests/reads.rs:489, tests/contents.rs:259). S01
   precedes S02 and skein-forge must not depend on the client (forge.md 2.1): the function (bounds.rs:205, over vocabulary types)
   moves with S01.2, or those asserts change.
3. T02.1 retargets temper-engine-protocol-forge onto skein_forge_client, but the component and the client do not depend on each other
   (forge.md 2.1). Interim only: S03.1/T03.1 must remove `Shared.client` (limits.rs:61) and the `client::worst_case` use.
4. S02.2 lists "Restore before a namespace refused": the design names no such refusal (gap 4).
5. The outline base is stale (top); the designs only exist at skein d120820.
