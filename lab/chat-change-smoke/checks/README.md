# First fake run: end-state evidence

Decision 276: M1 uses a real QA driver with everything behind temper fake.
Paid QA count: 0. These private Python 3.11+ standard-library helpers and
`end-state` command prepare the first end-state read. No completed QA run or
actual wire/native-process proof is claimed here.

`_project_observations(raw_bytes)` supplies `_judge`'s `complete_lines`,
`loss_counts`, `last_merged` and `last_moved` inputs. Its source contract is
skein `fbc771fde2a784d3b95e7a03460c4ccc56829c9d`,
`crates/skein-fake-forge-protocol/src/observations.rs`, `observations/read.rs`
and `observations/writer.rs`; see skein's `fake-forge.md`, 2.4 and 4.
It selects the last `fake/widget` merge with base `main` and main move in
writer order; deletion clears main until another move. It validates those
three record shapes and loss checkpoints, recognizes the other official
tags, and ignores their payloads. It does not implement the general codec
validator or promises judge. Content fields are base64, names UTF-8, and
counts exact u64 JSON numbers.

Pass still requires a complete newline-terminated stream ending in an explicit
zero-loss checkpoint, zero at every earlier checkpoint, the owner's confirmed
flush/stop boundary supplied separately as `flushed=True`, and actual object
evidence. A checkpoint establishes a complete written prefix; EOF, polling,
or perfect file content cannot establish that the owner reached the boundary.

Decision 278 supplies the launch and observation-read forms. Start the forge
with `temper fake forge --document F --ready PREFIX --seed N --observations FILE`.
Its ready stdout is PREFIX, then `127.0.0.1:PORT` and a newline, with no other
stdout. FILE is the official codec's stream; no reply or delivery of a pass
is sent before its observations are written. After the attended story, retain
the observed completion or stopped-writer evidence and the complete checkpoint
prefix being judged; no polling is needed.

The object decoder's authority is **Director280's 23:29Z provisional t18
contract**, explicitly authorized for this work before landing/diff-check.
Its referenced skein `118bdd2d` remains unavailable and unverified here; no
fetch or source-verification claim is made. Recheck the envelope, parent
encoding, base64 and partial-answer behavior against the landing hash's
`crates/skein-fake-forge-protocol/src/git/store.rs` (`objects`), `git/write.rs`
(`parents`) and `docs/design/fake-forge.md`, section 4, when relayed.

`POST /.fake/git/objects` takes **no credential**, with body `{"commits":[C,P]}`
using exact u64 JSON integers. Success 200 has this shape (member order is free):

```text
{"objects":[{"commit":N,"parent":N|null,"merge_parent":N|null,
             "tree":[{"path":B64,"content":B64}],"message":B64}],
 "graph":[{"commit":N,"parent":N|null,"merge_parent":N|null}]}
```

Null means absent parent. Paths, contents and messages are raw bytes encoded
as standard padded base64. `objects` holds whole asked commits in asked order;
`graph` holds only unasked ancestors, newest first through both parents.
Asked plus graph are bounded by `max_response_items` and answer room; excess
requested count refuses and an unknown commit is 404 Missing. Answer-room
exhaustion can drop a later asked commit from both lists, never move it to
graph. Two small commits fit. `_decode_objects` requires both asked C and P
whole in `objects`; graph-only or dropped C/P is unavailable. It rejects
malformed JSON/base64, non-integer counts, duplicate object IDs or decoded
paths, and missing object fields (including the message bytes it validates).
It does not validate the ancestry graph or turn missing trees into empty ones.

`end-state` first uses `_project_observations` and `_judge` with no objects.
Loss, truncation or unknown boundary stays undecided without store requests;
a complete known not-landed failure needs none either. Otherwise it reads C
with `{"commits":[C]}` to learn first parent P, then asks `{"commits":[C,P]}`.
It uses only the second response for the final C/P judgment, requiring both
whole there. A known C with no parent already fails the one-file judgment.

The host supplies the actual captured observation file and ready-line address,
an independent confirmed flush/stop boundary, existing run-owned evidence
directory and result path, and its absolute `time.monotonic()` deadline:

```sh
lab/chat-change-smoke/checks/end-state "$OBSERVATIONS" "$FORGE_ADDRESS" \
  --boundary confirmed --deadline-monotonic "$HOST_DEADLINE_MONOTONIC" \
  --evidence-dir "$RUN_EVIDENCE_DIR" --result "$RUN_CHECK_JSONL"
```

These are lab helper arguments. `FORGE_ADDRESS` is exactly `127.0.0.1:PORT`
from the forge ready line. Use `--boundary unknown` if completeness is not
confirmed; the flag itself cannot establish the boundary. The host retains
the actual boundary evidence. `NOTICE` is always the existing 22-byte file
beside `checks/`; there is no alternate expectation argument.

The host enforces the existing **120s total command cap**, including both
requests and file IO. Each socket operation uses time remaining before the
earlier of the host deadline and command start plus 120s; there is no fresh
per-request allowance. The command has no retries, polling, sleeps, redirects,
proxy use, admin read or Authorization header. No fake data is generated.

For each attempted request it preserves `objects-1.request.json` and original
`objects-1.response.body` bytes (then `objects-2.*` if needed), retaining
received partial bytes on read failure. HTTP statuses, C/P, paths and NOTICE
evidence go to stderr. Use a fresh directory per invocation. Status is 0 pass,
1 fail or 2 undecided, with skein's `lab.md`, 7.2 JSON lines on stdout:
silence on pass, stable `fail`/`summary` lines on failure, `undecided` on unknown.
`--result` stores those same lines, empty on pass; retain the exit status in
the ordinary host command record. No extra result schema is introduced.

The targeted check is run only through heavy:

```sh
heavy python3 -I -B -m unittest discover -s lab/tests/unit -p test_end_state.py -v
```

The existing 14 tests are preserved; two compact pure decoder tables add
binary trees/full u64/unordered members, exact NOTICE-only delta, and
partial/malformed evidence cases. Real fake and serve readiness, the confirmed
boundary and t18 landing review still precede actual first-run wire proof.

Keep ordinary per-run `run.json`, `driver-transcript.jsonl`, check result and
findings, plus raw observations and object request/response evidence. Record
the actual boundary evidence and any incomplete or refused read. These need
no new record format or lifecycle. Driver prompt/ctl guidance belongs to the
separate one-shot outside this worktree.
