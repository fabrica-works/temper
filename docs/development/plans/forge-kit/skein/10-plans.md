# Session 10: skein-forge's plan tests and the settled differences

Pin the moved plans on canned exchanges, then settle their differences from the design.
Needs at start: skein 01.4 (README, sessions).

## Read first

- skein `docs/design/forge.md` 3.2, 3.7, 5.2–5.4, 6–7, 11 (Plans, Marker, Deliveries);
  `testing-strategy.md` 2.1, 10.
- README (sessions, Rules, Done when); `code-organization.md` 5, 6, 8; `decisions.md` 28,
  Design questions 7–12 and 34–35; designer answers 7–12 and 35 (`forge-kit-answers-2.md`);
  `skein/01-forge.md` (01.3's plans, 01.4's `plans/tests.rs`); `skein/08-machine-world.md`,
  08.1 (10.5 only); findings `base-skein-01.md` (the draft's 01.5 estimates and "Answers 7–12 and 35,
  traced against the base"), `base-skein-10.md` (the split, per-test lines, rg checks).

## What the design asks

- 11: each plan against canned answers for every call of 5.2: the page size, `%2F`, 2554,
  each failure of 5.4 by status and message, the first failure ending a call, a request never
  seen and one that may have been (writes, `OpenPull`'s reads), the tables; the marker's words;
  each delivery of 7 to its hint.
- 5.2–5.4, answers 7–12 and 35: the differences between the plans and the design, settled.

## Rules that bind it

- This was the draft of skein 01's 01.5: about 1,100 new lines and five behaviour changes in one
  increment (decision 28). Every test and behaviour it had is kept with its name, values and
  budget; only the split changed, and what the split forces (a reference test where a file
  or a kind of test first appears; a default `Date` in the harness).
- 10.1 to 10.3 pin what the plans do now and change no production file. A behaviour change is
  its own increment's, with its own test (10.4, 10.5); nothing else changes behaviour.
- Tests use `kit`, never a system name (organization 5); no alias, re-export or serde. Each
  new test is a pure unit under 20 ms unoptimized: no Component, socket or clock. Bounds are
  the protocol fixtures': rows 8, answer 65_536, body 1045, op 512, page size 2 (cases 2/8/1).
- The tests are `forgejo::plans::tests::{calls,edges}`; the Proof counts filter on that path,
  since skein 06.3 adds skein-forge tests in its own order. Size counts a file with no base
  source as all new (organization 6); per-test lines are estimates (`base-skein-10.md`).
- Files stay under 800 lines and test functions under 40 (organization 8): a table of cases
  is data walked by a short loop.

## Increments

### 10.1 The plans' harness and the read calls
Scope: forge.md 3.2, 5.1–5.3 (page size, percent-encoding, 2554), 6 (revision); the harness.
Needs: skein 01.4.
Files: new `crates/skein-forge/src/forgejo/plans/tests/calls.rs`; `forgejo/plans/tests.rs`
  (`mod calls;`) and the shared `forgejo/tests/fixtures.rs` (Run) changed. No production file.
Shape: no production change. `fixtures::Run` (names are a sketch) owns a Plan, Tables and
  Bounds {page_size 2, rows 8, answer_bytes 65_536, body_bytes 1045}; `Run::known(op)` seeds
  REPO's row from `specimen`, `Run::new(op)` leaves the tables empty, `Run::with(bounds, op)`
  takes other bounds. `begin()` and `answer(status, body)` feed Heard: a head with a `Date`
  and no Retry-After, X-HasMore or Content-Range (`answer_with` names other Fields), then the
  JSON body decoded by the dialect's `fixtures::{decode,limits}` into Heard::Row and Done; a
  plan's Err comes back as Step::Done(Err(..)). `sent()` lists each Send as its request line,
  `METHOD target`, encoded by the dialect; `tables()` shows the Tables. No cost field: the
  component counts cost, one per request written (answer 34). The default head's `Date`,
  every resolve answering the id asked, and a PullFiles case with fewer rows than its page
  keep these five tests as they are through 10.4 and 10.5.
Test shape: 5 new tests in calls.rs, each <20 ms; the 19 cases are a table, walked by a loop:
  `read_calls_follow_canned_exchanges`: the 19 reads of 5.2, one case each, 5.2's request
  sequence (a resolve by id first where the row is unknown), literal Answer and literal sends,
  and `tables()` after each: REPO's row kept by the resolve, the login kept by Permission, the
  row dropped by a terminal 404;
  Job's log read is Range::Tail(4), File's Range::Head(4), both set explicitly;
  `user_search_uses_id_and_one_result`: `GET /api/v1/users/search?uid=7&limit=1`; absent ->
  Missing; `page_size_is_passed_to_each_listing`: the seven listings (Items, Reviews,
  Statuses, Remarks, Branches, Collaborators, PullFiles) at page sizes 2, 8 and 1, each
  `limit=` the size; `branch_slashes_and_2554_have_golden_targets`: a branch `a/b` goes out
  `a%2Fb`, and a `since` of u64::MAX nanoseconds as its literal UTC 2554 target;
  `long_body_revision_has_big_endian_golden`: Long(5), updated 3 ns -> 0x46c3_95ed_52b0_b56f;
  kept b"abc" -> 0xba78_16bf_8f01_cfea. References (the golden is the source's, the other
  shows Run):
```rust
// crates/skein-forge/src/forgejo/plans/tests/calls.rs
#[test]
fn user_search_uses_id_and_one_result() {
    let mut run = Run::known(Op::Read(Read::Permission { user: 7 }));
    run.begin();
    assert_eq!(run.sent(), ["GET /api/v1/users/search?uid=7&limit=1"]);
    assert_eq!(run.answer(200, br#"{"ok":true,"data":[]}"#), Step::Done(Err(Error::Missing)));
}
#[test]
fn long_body_revision_has_big_endian_golden() {
    let body = crate::forgejo::document::Body::Long(5);
    assert_eq!(super::super::reading::revision(&body,
        Some(skein_lib::Wall::from_nanos(3))), 0x46c3_95ed_52b0_b56f);
}
```
Proof: unit; `heavy cargo nextest run -p skein-forge -E 'test(/plans::tests::(calls|edges)::/)'`
  passes 5 tests, <1 s; no connected component yet (skein 03.1 brings component tests).
Don't: change a production file, give Run a cost field, a Component, a socket or a clock,
  rewrite a decoder, change a measurement (skein 02.2, 03.3), verify a signature, or put a
  system name in a value or comment.
Size: about 0 lines moved, 300 new: tests and fixtures only (estimated: calls.rs 230, Run 70).
Removes: nothing.

### 10.2 The write calls and the failure order
Scope: forge.md 3.2, 5.2, 5.4 (each row, the order, a request unseen and one seen).
Needs: skein 10.1's Run and calls.rs.
Files: new `forgejo/plans/tests/edges.rs`; `forgejo/plans/tests/calls.rs`,
  `forgejo/plans/tests.rs` (`mod edges;`), `forgejo/tests/fixtures.rs` changed. No production
  file.
Shape: no production change; the tests pin the moved failure table and write order. Run gains
  `fail(Failure)` (Unseen or Seen, how the component tells an exchange failed) and `answer_with`
  for a Retry-After. A write's Heard comes from the Run exactly as a read's.
Test shape: 9 new tests, 7 in calls.rs and 2 in edges.rs, each <20 ms; the 13 writes are a
  table walked by a loop:
  `write_calls_follow_canned_exchanges`: the 13 writes of 5.2, literal body, status, result and
  sends; CreateBranch's 409 is Branch(Exists), never Busy;
  `refusals_match_status_and_whole_message`: every 5.4 row, literal Error, Drop and retry; a
  message off by a byte is not named, and the status alone names none;
  `first_failure_ends_the_plan`: an Item read failing sends no comments request; sends 1 with
  the path known, 2 without (the resolve first);
  `unseen_write_is_unavailable`; `seen_write_is_timeout`: a failure before and after the first
  byte of a write;
  `open_pull_read_failure_is_unavailable`: each of its 3 reads (head, base, comparison) failing
  Seen or answering malformed ends Unavailable, no POST sent, sends 1, 2 and 3;
  `reviewers_are_added_before_removed`: wanted [7], current [8] -> POST ["seven"], DELETE
  ["eight"], in that order;
  `pull_files_stops_at_a_stale_head`: the pull request's head is not the call's -> Stale, one
  request; `job_mismatch_is_missing_job`: head, run or attempt mismatch, or a 404 -> MissingJob.
  edges.rs is new, so a Run reference (`fixtures::PULL`, contents.rs's test constant, number
  1, head 0x01.., copied into the fixtures; asked [0; 32]). Both files open with the `use`s
  of Run and PULL from the fixtures, the vocabulary's Op, Read and Error, and the plans' Step:
```rust
// crates/skein-forge/src/forgejo/plans/tests/edges.rs
#[test]
fn pull_files_stops_at_a_stale_head() {
    let mut run = Run::known(Op::Read(Read::PullFiles { number: 1, head: [0; 32], page: 1 }));
    run.begin();
    assert_eq!(run.answer(200, PULL), Step::Done(Err(Error::Stale)));
    assert_eq!(run.sent(), ["GET /api/v1/repos/fixture/specimen/pulls/1"]);
}
```
Proof: unit; `heavy cargo nextest run -p skein-forge -E 'test(/plans::tests::(calls|edges)::/)'`
  passes 14 tests, <1 s; no connected component yet (skein 03.1 connects it); base form is the failure and write tests of findings/base-skein-01.md.
Don't: change a status, a refusal or a drop, reorder the reviewers, count cost in Run, or
  test through a Component (the 68 component tests are skein 03.1's).
Size: about 0 lines moved, 270 new: tests and fixtures only (estimated: calls.rs 211, edges.rs
  42, Run 20).
Removes: nothing.

### 10.3 Deliveries, markers and the pinned statuses
Scope: forge.md 5.2 (statuses), 5.4 (Merge's read), 6, 7, 11; answers 11 and 35.
Needs: skein 10.2's edges.rs.
Files: `forgejo/plans/tests/edges.rs`, `forgejo/plans/tests.rs` (imports) changed. No production
  file: the base already does each of these.
Shape: no production change. Answer 11 and answer 35 settle the design, not the plans: the
  base already rejects another 2xx as malformed, and `writes::Plan::error` already turns every
  failure at `Stage::Merged` into Timeout, past `failure::classify`'s RateLimited. The source's
  `absent_optional_headers_use_the_fallback` is split in two, each half named for what it
  pins: its 429 half is `rate_limited_without_retry_after_waits_zero` here, its X-HasMore half
  is `pull_files_without_has_more_is_more_only_when_full` in 10.5 (that half changes
  behaviour). The first's doc comment says so.
Test shape: 8 new tests in edges.rs, each <20 ms; the first five call skein 01.3's pure
  `hint(forge, word, delivery)` and `marker::{valid_word, length, encode, decode}` (word first),
  no Run:
  `deliveries_map_subscribed_events_to_hints`: 22 events over 13 headers, section 7's literal
  Hints, a branch's deletion and the job ends included, a Long body's key None;
  `irrelevant_delivery_has_no_hint`: a tag push, a tag create and delete, a repository id of
  4_294_967_296 -> None;
  `markers_accept_words_at_both_bounds`: b"a" and b"abcdefghijklmnop" with a key of OP_BYTES
  (512): lengths 1040 and 1055 (kit: 1042); a limit equal to the length fits, one less fails;
  `malformed_word_is_refused`: empty, 17 bytes, upper case, a space, a colon -> false;
  `another_word_has_no_key`: kit reads a body marked `other:key` -> None;
  `success_statuses_match_each_exchange` (answer 11): reads 200, raw and log 200 or 206;
  writes 201, Review, Merge and Update 200, DeleteBranch 204, SetReviewers add 201 and remove
  204 succeed; any other 2xx is InvalidAnswer on a read, Timeout on a write, Unavailable on
  OpenPull's reads before its POST;
  `merge_read_429_is_timeout` (answer 35): POST 200 then GET 429 (Retry-After none, then 5 s)
  -> Timeout, sends 2; POST 429 -> RateLimited { after: 0 }, sends 1;
  `rate_limited_without_retry_after_waits_zero`: a 429 without Retry-After -> after 0.
  Pure-function reference, the source's:
```rust
// crates/skein-forge/src/forgejo/plans/tests/edges.rs
#[test]
fn another_word_has_no_key() {
    assert_eq!(crate::marker::decode(b"kit", b"<!-- other:key ab -->\n"), None);
}
```
Proof: unit; `heavy cargo nextest run -p skein-forge -E 'test(/plans::tests::(calls|edges)::/)'`
  passes 22 tests, <1 s; no connected component yet (skein 03.1 connects it); base forms are the hint, marker and writes tests of
  findings/base-skein-01.md. A 2xx that fails here is a plan defect: stop, do not edit it.
Don't: change a status table, a marker or a hint; touch `classify`'s generic 429; add a
  production line to make a test pass.
Size: about 0 lines moved, 160 new: tests only (estimated: edges.rs 156, imports 4).
Removes: nothing.

### 10.4 Bounds, room and ids (answers 7, 9, 12)
Scope: forge.md 5.3 (malformed), 5.4 (TooLarge, Timeout, Unavailable), 3.2 (File, PullFiles).
Needs: skein 10.3, whose tests pin the neighbouring behaviour.
Files: plans/{writes,reads,contents,actions}.rs and plans/reads/done.rs; edges.rs;
  `forgejo/tests/fixtures.rs` (Run).
Shape: three behaviour changes, each with its test. Show each against the base first. Run
  gains `with_limits(limits, bounds, op)` (the dialect's Limits, `inner` among them) and
  `hear(Heard)`, which feeds a Heard as given (a raw body, or a failure such as the `log`
  field's Bound); names are a sketch.
  Q7 (`write_inner_overflow_is_timeout`): a Malformed::Bound (a list past `inner`) is
  TooLarge on a read and Timeout on a write. writes.rs's arm gives `TooLarge` through `error`
  now: give `if before_write() { TooLarge } else { Timeout }`, so OpenPull's reads before
  its POST keep TooLarge (a bare Timeout would become Unavailable); reads.rs, contents.rs and
  actions.rs map the same variant to TooLarge where they give InvalidAnswer. The client's own
  admission of a write's answer past `answer_bytes` stays InvalidAnswer (skein 02.2). The
  log read-through (`field: b"log"`) is a body past `body_bytes_in`, which forge.md 5.3 keeps
  malformed: the read arm tests the field, so it stays InvalidAnswer (decision table, row 7).
  Q9 (`file_cuts_to_remaining_answer_room`): contents.rs `raw` returns TooLarge for File when
  the room left is below `max`. Cut to min(max, answer_bytes - path.len()) and set `truncated`
  when cut, as Job's tail after its step name does; drop the `more && length < max` arm for
  File. A side of PullFiles or Compare is never cut: it stays TooLarge.
  Q12 (`repository_id_mismatch_is_invalid_answer`): reads/done.rs accepts any nonzero id from
  `repositories/{id}`. Compare the id with the one asked, for RepositoryById (resolve and
  Settings) before `keep_document` and `keep_repository`, in reads and in writes, so a mismatch
  caches no row and no owner. Read -> InvalidAnswer; write -> Timeout, OpenPull before its
  POST -> Unavailable (through `error`); an id past 32 bits stays TooLarge, checked first.
  Locate asks by path and still accepts any nonzero id.
  temper's side: none; no temper test expects the old behaviour (findings/base-skein-01.md,
  "Answers 7–12 and 35": rg and the named component tests' bodies).
Test shape: 3 new tests in edges.rs, each <20 ms, each a Run on its call:
  `write_inner_overflow_is_timeout`: limits with inner = 2, an answer of 3 labels ->
  Timeout after a POST; a read of the same overflow -> TooLarge; OpenPull's read before its
  POST -> TooLarge; a Job read failed with the `log` field's Bound -> InvalidAnswer;
  `file_cuts_to_remaining_answer_room`: path b"x", answer_bytes 4, max 4, fed b"abcd" ->
  b"abc" with truncated; a Job with step name b"x" has raw_room 3, fed b"bcd" and more ->
  b"bcd" with truncated (pinned already); the same short PullFiles and Compare sides ->
  TooLarge;
  `repository_id_mismatch_is_invalid_answer`: asked 7, answered 8 at resolve and at
  Settings -> InvalidAnswer, no row and no owner in the tables; a write -> Timeout (OpenPull
  before its POST -> Unavailable); Locate -> Located 8.
Proof: unit; `heavy cargo nextest run -p skein-forge -E 'test(/plans::tests::(calls|edges)::/)'`
  passes 25 tests, <1 s; no connected component yet (skein 03.1 connects it); the 22 of 10.1 to 10.3 pass unedited.
Don't: change skein 02.2's admission, the generic 429, a decoder or a limit; cut a PullFiles or
  Compare side; make Locate check its id; fold the three changes into one commit.
Size: about 0 lines moved, 240 new: 135 production plus 105 tests and fixtures (estimated:
  Q7 30, Q9 40, Q12 65; edges.rs 86; Run 15; imports 3).
Removes: the differing bound, room and id behaviours.

### 10.5 Paths and headers (answers 8, 10)
Scope: forge.md 5.2, 5.3 (absent headers), 5.4 (a 404 that ends nothing); forge-connection 6.
Needs: skein 10.4; skein 08.1, which moves the machine world to skein.
Files: plans/{contents,actions,reads}.rs; edges.rs; skein
  `tests/forge/machine/src/fixtures.rs` and `tests/forge/machine/tests/machine.rs`.
Shape: two behaviour changes, each with its test.
  Q8, B (`answering_404_keeps_the_path`): contents.rs and actions.rs drop the path before
  they ask whether a 404 is an answer. Decide first: a missing side of PullFiles or Compare,
  and Checks' missing run or job, keep it; every terminal 404 (File's, Job's MissingJob) and
  every 3xx still drop it, as reads.rs already does.
  Q10 (Date: `items_without_date_is_invalid_answer`; X-HasMore:
  `pull_files_without_has_more_is_more_only_when_full`): reads.rs takes `now` from
  Items' own listing head and returns InvalidAnswer without a `Date`, even after a dated
  resolve (it uses Time::ZERO now); other reads keep ignoring it. contents.rs defaults a
  missing X-HasMore to false: use the page's fullness when absent, an explicit false staying
  false. A 429 without Retry-After is `RateLimited { after: 0 }` already (10.3).
  Machine world (skein 08.1 moved it): the undated Items head in
  `a_body_past_body_bytes_is_carried_empty_without_its_key` would now be InvalidAnswer. Add
  `fixtures::dated_response(code, body)` (a `Date: Thu, 01 Jan 2026 00:00:00 GMT` line before
  Content-Length) and script that one head with it. Not in `response`: fuzzy_machine's
  byte-cut sweeps run 7.5–7.7 s against a 7.8 s ceiling, and a Date line in its three heads
  would lengthen them. skein-http supplies a Date on the wire, so no Reply changes. temper's
  side: `tests/engine/protocol-forge` (`src/fixtures.rs::response`, `tests/machine.rs`) has the
  same test; temper's next repin repairs it while temper has it (temper 07 deletes it).
Test shape: 3 new tests in edges.rs, each <20 ms; no machine test is added or removed:
  `answering_404_keeps_the_path`: a missing PullFiles or Compare side and Checks' missing run
  or job -> path in the tables; a terminal Job 404 -> MissingJob, path dropped; a 302 ->
  Unavailable, dropped;
  `items_without_date_is_invalid_answer`: a dated resolve, then an undated Items head ->
  InvalidAnswer;
  `pull_files_without_has_more_is_more_only_when_full`: page size 2, 1 and 2 rows with no
  X-HasMore -> more false and true; an explicit false with 2 rows -> false.
Proof: unit; `heavy cargo nextest run -p skein-forge -E 'test(/plans::tests::(calls|edges)::/)'`
  passes 28 tests, <1 s; no connected component yet in skein-forge itself;
  world; `heavy cargo nextest run -p skein-forge-machine-world` passes the count it had at
  the step's start, each <0.5 s: 33 after skein 08.1, then 38, 40, 41 and 42 after 08.2 to
  08.5 (none added or removed here); only that world scripts an Items head without a Date.
Don't: put a Date in `response`; add one to the fake's Reply; touch a terminal 404's drop;
  take `now` from a read other than Items; change a test's expected values to pass.
Size: about 0 lines moved, 165 new: 85 production plus 80 tests and fixtures (estimated:
  Q8 60, Q10 25; edges.rs 66; world repair 12; imports 3).
Removes: the differing 404, Date and header behaviours; the undated scripted head.

## Done when

- `heavy cargo nextest run -p skein-forge -E 'test(/plans::tests::(calls|edges)::/)'` prints 28
  passed, 0 failed, <1 s in all; each test <20 ms.
- `heavy cargo nextest run -p skein-forge-machine-world` prints the count skein 08 has
  reached passed (33 after 08.1, 42 after 08.5), 0 failed, each <0.5 s.
- `heavy cargo clippy -p skein-forge --all-targets -- -D warnings` exits 0, prints Finished
  and no warning.
- `heavy rg -n -i temper crates/skein-forge tests/forge/machine` prints nothing (exit 1).
