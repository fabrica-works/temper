# The fake forge's jobs and the repair's brief, at the base

The code audit of temper at `118f7fb7` (`origin/main`), 2026-10-16, by a
read-only sub-agent. Paths are temper's.

## 1. The fake's runs, jobs and logs

- `Run { id, number, commit }` and `Job { id, run, index, attempt }`
  (`testing/temper-fake-forge-domain/src/actions.rs:10-22`): one job per
  configured context, attempt 1 (`:29-41`); a re-run sets attempt 2
  (`:43-48`); another attempt answers `Missing(Job)` (`:80-90`).
- Reads `Runs`, `Run`, `Jobs` (`reads.rs:94-96`) and the log,
  `Read::Job` (`reads.rs:67`), served by `next.rs:150-192`: only for a
  failed status the CI user wrote (`:164`); the log is four synthetic
  lines, `failing check <context> <0..3>\n` (`:175-188`), of which the
  last `max_bytes + 1` bytes are answered (`:189-191`), so the fake cuts
  the log's end before any range applies.
- **No job by its id and no steps:** `crates/temper-forge-forgejo/src/server/actions.rs:24-44`
  routes `runs`, `runs/{id}`, `runs/{id}/jobs` and
  `jobs/{id}/logs?attempt` only; `request.rs:132-150` has no `Job`
  request, and `JobLog { at, job, attempt }` no `step`; a job is written
  as `id`, `run_id` and `attempt` (`server/actions.rs:76-89`). A new route
  also goes in `tally.rs:31-32` (the route map is sized "39 routes",
  `server.rs:432`).
- **Where steps can come from:** `Checks` (`api.rs:751-765`) names
  contexts only. A fixed trio per job is the cheapest: 0 `Set up job`,
  1 one workflow step, 2 `Complete job`, `failure` whenever the job
  fails. "Only the completion failed" and "no step failed" need a
  seeded choice per context. A silent context never fails today, and
  `next.rs:164` needs a failed status.

## 2. The fake's ranges

- `range` (`testing/temper-fake-forge-protocol/src/response.rs:426-443`)
  parses `bytes=F-L` and `bytes=F-`; `bytes=-N` is refused
  (`id::unsigned("")` is none, `id.rs:15-18`), and
  `src/tests/response.rs:158` asserts the refusal. A parse error fails
  `select` (`server.rs:381,448-451`).
- `raw` (`:444-486`): `416` when `first >= total`; else `206` with
  `content-range: bytes F-L/T` (`:463-470`); `TooLarge` above
  `raw_bytes` (`:482`). Since the domain already cut the log
  (`routes.rs:217` passes `raw_bytes`), `T` is the cut body's length,
  not the log's.
- `step=` is not parsed (`server/actions.rs:37-43`), encoded
  (`client/files.rs:100-104`) or passed (`routes.rs:217`).

## 3. The component's log read

- The component sends `Range: bytes=0-<max>`
  (`crates/temper-engine-protocol-forge/src/connection.rs:180-188`),
  reads Run, Jobs, JobLog (`actions.rs:269-302`), keeps the first `max`
  bytes (`actions.rs:350-362`), and computes `truncated` as `total >
  last + 1` (`actions.rs:139-146`). So over HTTP a brief gets the start
  of the fake's already-cut end; in the domain worlds
  (`tests/engine/forge/client/src/translate.rs:498-508`) it gets the
  last `cap` bytes.

## 4. The brief

`crates/temper-engine-domain-forge/src/brief.rs`:

- Its head (`:506-508`): `Repair the failed check at change head
  <hex8>. Push a new head and wait for checks there.` It reads
  `Checks { head }` (`:531`); for each failed status (`:376-407`):
  `Failed check: `, `Description: `, `Link: `; then `Read::Job` when the
  status names a job at this head, else `[no pinned job log for this
  head]` (`:403`).
- The answer (`:306-317`): `Job log tail:\n<log>\n`, then `[job log
  truncated]`; any error or mismatch, `[job log could not be read]`. It
  ends `Repair the failed check.` (`:302`, `:320`).
- The share: `bytes / every check (failed or not) / 2`, at least 1, at
  most `max_job_bytes` (`:394-401`); `max_job_bytes = min(answer_bytes,
  ci_budget / 5)`, at least 1 (`domain.rs:700-703`). Bytes past the
  section's room count in `left` (`:61-69`, `:79-91`).
- No step: `Answer::Job` has none (`crates/temper-engine-domain-forge-client/src/api.rs:156`).

## 5. The tests that grow

- `tests/engine/domain/tests/forge_routes.rs`:
  `a_failed_ci_repair_brief_contains_the_bounded_job_log` (`:1178-1241`,
  64 lines) and `an_unreadable_failed_job_log_is_named_and_repair_still_runs`
  (`:1244-1294`, 51 lines), on `change_world(false, 0, None)` (`:633`)
  and `configured_policy` (`:78-311`: CI budget 384, `brief_bytes` 768,
  `org/repo`, context `build`, `passes = 0`). The first asserts `failing
  check build 3` and `[job log truncated]` (`:1233-1240`); the second
  sets `fail_job_reads` (`:1246`, answered `MissingJob`, `:414-425`).
  Helpers: `World::external` (`:363-390`), `slow_brief_reads`
  (`:436-444`), `failed_ci_with_delayed_brief` (`:1296`).
- The fake domain's: `failing()` (`testing/temper-fake-forge-domain/src/tests/actions.rs:4-13`),
  the log table (`:73-95`), `src/tests/next.rs:167-189`.
- The fake protocol's: `src/tests/response.rs:129-165`,
  `src/tests/routes.rs:101-105`; `tests/codecs/tests/forgejo_files.rs:37`.
- The component's job tests run on canned answers only:
  `crates/temper-engine-protocol-forge/src/tests/contents.rs:406-476`.

## 6. Worlds above the domain world

- `tests/system/src/forge/stories.rs:209-326` runs a failed CI, a repair
  and a landing (`repairs == 1`), asserting only `failing check build 3`
  (`:256-262`), through `translate` (`tests/system/src/forge.rs:329-419`),
  not HTTP.
- `tests/one-process`, the only world on the fake's HTTP layer
  (`forge.rs:10`), never fails CI: `CI_PASSES = 1000`
  (`fixtures.rs:41`), `cue: None` (`stories.rs:152-154`).
- The process tier (`crates/temper/tests`) and
  `tests/engine/protocol-forge` run no failed CI.
