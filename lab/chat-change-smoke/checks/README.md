# First fake run: end-state evidence

Decision 276: M1 uses a real QA driver with everything behind temper fake.
Paid QA count: 0. These private Python 3.11+ standard-library helpers prepare
the end-state read; there is no executable checker or completed QA run here.

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

Await the director's exact launch/observation-read forms and the first-run
owner's completed flush/stop evidence, identifying the captured observations
file and boundary. Also await the supplied fake private git `objects` read
form and listener: how to request the selected C and its first parent P and
obtain their complete parent/merge-parent values and byte trees. The route
can return graph-only ancestors, so P must be requested whole too. No CLI,
configuration or credential inputs are guessed; unavailable objects remain
`None` for `_judge`.

Keep ordinary per-run `run.json`, `drivertranscript.jsonl`, check result and
findings, plus raw observations and object request/response evidence. Record
the actual boundary evidence and any incomplete or refused read. These need
no new record format or lifecycle. Driver prompt/ctl guidance belongs to the
separate one-shot outside this worktree.
