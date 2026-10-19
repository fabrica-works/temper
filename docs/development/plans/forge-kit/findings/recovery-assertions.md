Historical follow-up artifact for skein02.s2; the landed supplement and relative recount govern implementation. The snapshot audit below was prepared before/while02.s1 landed.

# Findings 2–6: assertion and case map

Base: skein 6e29079218b9aa0bced12e8fe62b6c1ea02fb2c6 (landed local main; origin/main recorded separately). All names were checked in source, no graph consulted. No ownership transfer from RUNNING contract-only 02.s1.

## Original cases

| Old behavior/cases | Destination | Retained assertions |
|---|---|---|
| oversized_hints_read_nothing: branch, key | same two rows, each now carries Answer::Commit([1;32]) or empty_item() | Fact::Refused then None, identical deadline, no send, diagnostic name only |
| remake_releases_a_held_entry: precondition sequence | uncertain_comment_becomes_held_at_its_deadline | exact [(1, Uncertain), (1, Held)] sequence |
| remake_releases_a_held_entry: Remake saved uncertain entry after Held | same test using held_comment() -> (Harness, original saved Entry) | next op Write(comment), writes == 2, last Made Commented(9), found:false; durable-held insertion preserved |
| restore_refuses_bad_pages: comment zero, comment overflow, review zero, review overflow | same four rows | failed true; Keep Busy; failed instance still true |
| restore_refuses_bad_stamps: comments overflow/repeated; reviews overflow/repeated | same four rows | same failure/Busy/persistence checks |
| restore_refuses_oversized_caches: item bytes/rows; pull bytes/rows | same four rows | same failure/Busy/persistence checks; item labels count remains five |
| restore_refuses_oversized_echoes | same one case | same failure/Busy/persistence checks |
| restore_refuses_unpaired_rows: missing repository, missing live | same two rows | same failure/Busy/persistence checks |
| restore_refuses_capacity_overflow: resources, repositories | same two rows | same failure/Busy/persistence checks; no new table saturation |
| restore_refuses_invalid_watches: empty branch, long branch, participating repository, participating branch | same four rows | same failure/Busy/persistence checks |
| failed_restore: fresh generic paired record repeated for all 21 rows above | fresh_paired_restoration_accepts_keep -> accepted_live | !restoration_failed and Keep Ok(()), identical generic records; old row labels changed diagnostics only |

The 21 old fresh successes all constructed `Harness::new(LIMITS); paired(live()); Restored(Monotonic)`, with no rejected input reused. They are one identical assertion case, now independent, rather than 21 different corrected fixtures. The old failed-instance assertion stays with each rejection after its Busy check. `keep_is_busy_around_failed_restore` independently retains correction-in-the-same-instance and successful-Read persistence; it is not the generic fresh success.

Independent coverage check: the full client test tree's Restored/restoration_failed consumers were inspected. `contracts::binding_replaces_the_namespace_before_restore` and `tests::entry_refusal_is_observable_even_without_diagnostics` restore no paired records and assert different behaviors; `outbox::a_rehanded_entry_with_a_started_attempt_finds_before_writing` checks saved-attempt recovery; keep/outbox Harness initialization and fixture::ready set up restored domains. None independently asserts both generic paired-record restoration success and Keep Ok. Hence one independent generic success is warranted; no success per rejected cause is added.

The other 15 original recovery test functions retain every case/assertion, with deadline literals replaced by value-identical named fixtures. `assertion-inventory.md` and `assertion-map.json` list every original assertion location, including unchanged helpers. Original source files and the unified patch are retained.

## Exact-bound acceptance / refusal pairing

All records below are Stored::Live + the same Stored::Repository, finalized with Restored(Monotonic). Each accepting case asserts !restoration_failed and valid-sized Keep Ok(()); each refusal asserts failed, Keep Busy, failed still true.

| Accepting record | Precise one-past refusal | Test destinations |
|---|---|---|
| comment_page=LIMITS.inbox=16 | comment_page=17 | restore_accepts_exact_page_and_stamp_bounds / restore_refuses_bad_pages(comment overflow) |
| review_page=16 | review_page=17 | same accepting table / bad_pages(review overflow) |
| 16 distinct comment Delivery ids | 17 distinct ids | same accepting table / restore_refuses_bad_stamps(comments overflow) |
| 16 distinct review Delivery ids | 17 distinct ids | same accepting table / bad_stamps(reviews overflow) |
| cached item labels: four empty labels | five empty labels | restore_accepts_exact_cache_and_echo_row_bounds / restore_refuses_oversized_caches(item rows) |
| cached pull reviewers: four id=7 elements | five id=7 elements | same accepting table / oversized_caches(pull rows) |
| four Echo::Comment(1) elements | five elements | same accepting table / restore_refuses_oversized_echoes |
| cached item: title 4096 bytes, body/labels empty, key None | title 4097 bytes, other fields identical | restore_accepts_exact_cached_byte_bounds / oversized_caches(item bytes) |
| cached pull: head 4092 bytes + base b"main" (4), no reviewers =4096 total | head 4093 + base 4 =4097 total | restore_accepts_exact_cached_byte_bounds / restore_refuses_exact_one_past_cached_bytes |

The old pull bytes refusal uses head=4097 + base=4 =4101. It is preserved in addition to the exact-one-past refusal; it cannot be mislabeled as the precise bound pair. Summary accounting includes labels' STRING_BYTES widths; four/five empty labels plus title b"goal" use 68/84 bytes, both safely below 4096. Pull reviewers' ID_BYTES widths plus b"topic"/b"main" use 41/49 bytes. Echo checks only row count; no new production accounting is involved. Zero page and duplicate stamps remain independent refusal cases, not upper-bound pairs.

## Names and derivations

Fixture LIMITS in src/tests.rs: rows=4, inbox=16, answer_bytes=4096, op_bytes=256, resources=4, repositories=2, lifetime=10s, clock_margin=1s, hinted=1s. Shared Harness starts at100.
START_AT=100; HINT_AT=101=100+hinted; RETRY_AT=102=100+RateLimited(after2s); ATTEMPT_DEADLINE=111=100+lifetime+margin; RETRY_DEADLINE=113=102+lifetime+margin; RESTING_AT=160=100+resting_echo.poll60s. Named literal expected times are independent of the production deadline computation; comments cite derivations.
EXCESS_ROWS=LIMITS.rows+1=5; EXCESS_INBOX=17; EXCESS_ANSWER_BYTES=4097; EXCESS_OP_BYTES=257; EXCESS_RESOURCES=5; EXCESS_REPOSITORIES=3. PULL_HEAD_BOUND=4096-b"main".len()=4092; the exact one-past length is PULL_HEAD_BOUND+1=4093.

## Counts and consumers

Recovery 24 -> 30 test functions (+6). No original function removed; one assertion moves to Held's new test. New functions: Held precondition (+1), generic fresh success (+1), exact page/stamps (+1, four rows), cache/echo rows (+1, three rows), cached bytes (+1, two rows), precise one-past pull bytes (+1, one row). New acceptance boundary records=9, new precise refusal record=1. Baseline source has 114 test attributes; sketch120 is not the post-02.s1 count. Landed post-02.s1 total N becomes N+6, preserved names unchanged and new names +6. Distinguish process/test-function counts from table-row assertion cases.
The correction collapses 21 identical generic fresh success observations to one independently named case, while retaining all 21 failure/Busy/persistence cases. Additional exact refusal becomes the22nd rejected-record row. This is an assertion-case-preserving mapping by identical input/output, not deletion of a rejection cause.

Full repository sweeps are retained in names-consumers.txt, counts-consumers.txt, derived-consumers.txt. Only skein's recovery.rs/fixture helper owns these named behaviors; jig's similarly named restore readiness test is a different domain. Downstream temper restart/memory restores are existing consumers, not duplicate versions of these tests. No production API, value, limits or dependency pin changes, so no behavior repin is required by this supplement alone. Closing temper repin must wait for s2 as it waits for s1. Final count consumers are skein/02-client.md Done when (currently130/74+56 three lines); revise relative to observed s1 count. Its historical02.4 proof114 and all s1 proof numbers stay historical. README/session count/dependency registrations need supervisor ruling; do not rewrite the running s1 scope. findings/contracts-assertions.md count-sweep paragraph is historical, with a new s2 findings map cross-reference if desired.

Later source observation (no change by this investigation):02.s1 landed at532a14b8d3de750432a168e895c2e7bf0da81c02 during the follow-up. Both recovery files are byte-unchanged from6e290792 (post-s1-recovery-diff.txt); primary test attributes counted130 at that commit. N+6 remains the relative final obligation, with a new count at implementation time. The scratch source remains frozen to6e290792 plus this recovery proposal; it does not claim a successful post-s1 build. The landed scripts/check-contracts.rs bounds itself to four contract files and has its own tokenizer; the separate recovery AST checker preserves that ownership. gate-wiring.patch is now based on the landed s1 gate, leaving its four-file checks intact.
