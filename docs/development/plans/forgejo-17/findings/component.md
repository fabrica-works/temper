# The forge component's job read and comparisons, at the base

The code audit of temper at `118f7fb7` (`origin/main`), 2026-10-16, by a
read-only sub-agent. Paths are temper's.

## 1. Requests

- `crates/temper-forge-forgejo/src/request.rs:122-126`, `Compare { at,
  before, after }`; `:138-150`, `Run`, `Jobs`, `JobLog { at, job,
  attempt: u32 }`: no step, no comparison flags, no job by its id.
- The encoder: `client.rs:48-54` → `client/files.rs`: `:73-79` Compare
  as `/compare/<b>...<a>`, no query; `:95` Run; `:96-99` Jobs;
  `:100-104` JobLog as `/logs?attempt=`. Queries are written by hand
  (`out.put(b"?x=")`), with `client/items.rs:41-66`'s helpers (`repo`,
  `numbered`, `page(page, query: bool)` choosing `?` or `&`). A target is
  measured, then written (`files.rs:56-65`).
- A new `Request` variant adds an arm to: `client.rs:93-128`,
  `crates/temper-engine-protocol-forge/src/connection.rs:645-686`,
  `reads.rs:658-699`, `temper-forge-forgejo/src/client/{start.rs:36,106;
  writes.rs:123,245; reviews.rs:125; items.rs:151}`,
  `server/writes.rs:279,390,434`, and the fake forge's protocol
  (`routes.rs`, `answers.rs`, `response.rs`, `server.rs`, `tally.rs`).
- The server side: `server/actions.rs:24-45` (JobLog's arm `:37-43`
  reads `attempt` only; no `[jobs, job]` arm); `server/files.rs:30-38`
  ignores Compare's query, so flags would not survive the round trip
  that `tests/codecs/tests/forgejo_files.rs:182-186` checks.
- **The range:** `crates/temper-engine-protocol-forge/src/plan.rs:5-8`,
  `Exchange { request, range: Option<u32> }` (`Step::send`, `:16`, sets
  none). Producers: `actions.rs:299-302` (the log), `contents.rs:102-105`
  (a file), both `Some(max)`. The consumer, `component/calls.rs:219`,
  calls `Connection::start(range)` (`connection.rs:161`), which writes
  `bytes=0-<max>` (`:180-188`: max + 1 bytes, so one more shows more
  came) and sets `raw_max` (`:192`). File reads keep `bytes=0-N`.
- **Pinned request bytes:** `tests/codecs/tests/forgejo_files.rs:53-72`
  (compare, compare-reverse), `:111-114` (actions-run, actions-jobs,
  actions-job-log, actions-job-log-range), checked equal to the
  transcripts (`:172-180`) and routed back (`:182-186`); the transcripts'
  `request` files (the range case sends `Range: bytes=0-3`), recorded by
  `tools/forgejo-conformance/src/cases.rs:195-196` and `runner.rs:93-105`
  (the recorder's range is `Option<(u64, u64)>`, written `bytes={s}-{e}`
  by `testing/temper-throwaway-forgejo/src/http.rs:17`); the component's
  request lines in `crates/temper-engine-protocol-forge/src/tests/`:
  `contents.rs:143` (the reverse comparison's line), `writes.rs:295`
  (OpenPull's comparison), `contents.rs:410,471` (the log's line),
  `:476` (`Range: bytes=0-4`), `connection.rs:208-212`.

## 2. Documents

- A run decodes whole, not as rows: `temper-forge-forgejo/src/client/actions.rs:22-30`
  (fields), `:109-133` (scalars into `self.run`), `:97` (`Document::Run`).
  Jobs are a top-level array of `ActionKind::Job` (`:45-51`), each
  object ending in `Row::Job` (`:78-80`), the listing in `Listed {
  skipped }` (`:99`). Structs at `document.rs:326-340`; `Row` `:67-81`,
  `Document` `:83-102`: no job document.
- A job at the root is unreachable (`actions.rs:100`), and so are text
  scalars (`:131`): a step's `name` and `status` cannot be read yet.
- The closest pattern for a root with scalars and an array of objects:
  `FileKind::Compare` (`client/files.rs:37-45`), ending in
  `Document::Compare`; `ActionKind::Listing` (`actions.rs:34-41`).
  Required fields are enforced at `decode/walk.rs:422-423`.
- **The comparison's decoder requires both `commits` and `files`**
  (`client/files.rs:40-42`): if `files=false` leaves out the top-level
  `files`, those comparisons fail `Malformed::Missing`.

## 3. The job read

- `crates/temper-engine-protocol-forge/src/actions.rs`: stages `:14-26`;
  `begin` (`:105`) sends Run; the run (`:269-283`) checks head and run,
  then sends Jobs; each job row (`:223-229`) sets `found` by `run_id`,
  `id` and `attempt`; the listing (`:284-303`) answers `MissingJob` or
  sends the log with `Some(max)`. The head (`:136-146`) admits `206` at
  the log only, `truncated = total > last + 1`; `log()` (`:350-363`)
  keeps the first bytes.
- `first` is there: `header.rs:4-9`, `ContentRange { first, last, total
  }`, parsed at `:19-38`, carried in `Fields.content_range`
  (`connection.rs:33,428-438`).
- **A `200` past its cap:** `connection.rs:457-468` caps the buffer at
  `min(raw_max + 1, raw_cap)`; `:354-371` answer `Body::Raw { more: true,
  bytes }` with the first bytes and close (`Phase::Closed`), the rest
  unread; `:384` sets `more = len > max` at the end. The end is lost.
  Keeping the last bytes belongs in the `Receiving::Raw` arm
  (`:354-371`), with a flag in `Receiving::Raw` (`:87`) set by `start`
  (`:192`); `raw_demand` (`:476-485`) must go on reading when the buffer
  is full. File reads (`contents.rs:326-349`) keep the head.

## 4. Tests to extend

- `crates/temper-engine-protocol-forge/src/tests/contents.rs`: ATTEMPT,
  RUN, RUNS, JOBS, FAILING (`:302-306`), `failing()` (`:307-316`);
  `a_job_log_at_another_head_is_missing_job` (`:404-419`),
  `a_job_without_its_id_and_attempt_is_missing_job` (`:421-436`),
  `a_job_read_answered_404_is_missing_job_at_each_exchange` (`:438-450`),
  `a_jobs_verified_log_uses_range_and_content_range_reports_more`
  (`:452-486`); the Checks tests (`:317-402`) keep the jobs listing;
  `comparisons_read_both_directions_and_both_file_sides` (`:116-144`).
- `tests/writes.rs`'s `open_pull_*` (`:283-345`); `tests/connection.rs`'s
  `raw_reads_keep_the_range_and_refuse_bytes_past_remaining_answer_room`
  (`:205-227`, `raw_request()` `:237`).
- Helpers, `tests/fixtures.rs`: `known()` `:463`, `ok()`/`response()`
  `:351-371`, `status()` `:373`, `request_lines` `:222`, `request_bytes`
  `:241`, `ConnectionHarness::answer` `:425`.
- `crates/temper-forge-forgejo/src/tests.rs` decodes no job, run or
  comparison; related: `a_content_range_reads_its_first_last_and_total`
  (`:317-331`), `a_job_link_of_another_shape_names_no_job` (`:369-386`).
- The codecs world: `tests/codecs/tests/forgejo_files.rs` (`cases()`
  `:39-116`, `run()`/`jobs()`/`log()` `:17-38`, the mutation test
  `:204-232`, `a_failing_actions_status_links_its_run_and_job`
  `:234-259`); `fuzzy_forgejo.rs` routes every transcript.
- The conformance's component facts: `tools/forgejo-conformance/src/component/reads.rs:105`
  (the job) and `:67` (the comparison).
