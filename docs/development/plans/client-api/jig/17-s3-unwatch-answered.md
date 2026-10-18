# Supplement jig-17.s3: an unwatch of an ended watch is answered

Added by ruling 372 (temper-05.s2's block of 2026-10-18T14:23:16Z). Read
first: jig's `AGENTS.md`; `docs/design/protocol/client.md`, 7 ("Ending:
… or closing it by an exchange, answered once the engine has ended it. A
stream that ends ends its watches.").

Scope: `jig-protocol-engine-clients` answers every unwatch exchange. Today
an unwatch that comes after its watch's `WatchEnded` gets no answer. The
cause, from temper-05.s2's `logs/temper-05/s2-unwatch-cause.md`, at jig
`38ae1ba17`, the same at `e444994d4`, in `src/watches.rs`:
- `watch_route` (`:80-98`) sets `watch.closing` on an ended watch;
- `watch_ended` (`:352-354`) returns once `watch.end` is set, so no
  second ending answers it;
- `stream_up` (`:434-445`) removes the ended watch at `StreamGone` and
  answers `watch.opening`, but not `watch.closing`.
Needs: nothing. It runs beside jig 19.5 and jig 20, off their chain.

Shape: the closing of an ended watch is answered `Closed`
(`routes::close_answer(false)`, `routes.rs:76`) once its ending is
released, and at the latest when its stream goes, as an opening is.
Touch nothing else in `watches.rs`, and leave `stream_open` alone: the
designer's per-client stream answer (`e444994d4`) changes it in another
step.

Test shape, in `src/tests/watches.rs`, beside
`unwatch_answers_only_after_its_released_ending` (`:142`), which passes
unchanged:
- admitted, `WatchEnded(Gone)`, then the unwatch, then `StreamGone`: the
  unwatch is answered `Closed`, once;
- admitted, `WatchEnded(Gone)`, then the unwatch, then the ending
  released: answered `Closed`, once.

Proof: those two tests, the crate's suite, then jig's gate,
`scripts/check.sh`.

Size: about 30 production lines. Past 100, it asks.

Don't: change skein's read half-end (`crates/skein-http/src/server.rs:886-903`)
or any deadline.
