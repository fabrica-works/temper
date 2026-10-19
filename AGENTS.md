# Agent guidance for temper

- Before merging anything to main, run the checks in
  `docs/development/workflow.md`; main moves only when they all pass. A
  change to Markdown files only skips them (same document).
- **A plan's run lands through skein's `land`,** which runs those checks,
  `scripts/check.sh`, as its gate and pushes to `origin`'s `main`
  (workflow.md, section 1). A change to Markdown files only lands with
  `land --docs`. Outside a run, push only when the user asks.
- The default test suite (unit tests and the worlds' focused tests under
  `tests/`) takes at most 15 seconds; the fuzzy suite (the worlds'
  `tests/fuzzy_*.rs`, randomized tests) at most 1 minute. Keep new tests
  within these budgets: see the same document.
- temper is built on skein, the io and generic-protocol kit, and follows
  the foundation documents in the `docs/foundation` directory of skein's
  repository. temper's code and documents cite them by file name:
  - `programming-model.md`: how code is written; every crate follows it;
  - `testing-strategy.md`: how that code is tested;
  - `notes.md`: the questions still open, and why the memory strategy is
    what it is.
- temper's own design documents are in `docs/design/`, whose `README.md`
  comes first and says where each part lives; `domain/testing.md` is how the
  testing strategy applies to temper. Code cites a design document by its
  path from `docs/design/`: `domain/forge.md, 8.2`, `protocol/forge.md, 3.2`.
- jig, the kit temper is built on, is its own repository
  (`~/src/rust/jig`), pinned as skein and smith are
  (`docs/development/local-jig.md`). temper cites its documents as
  jig's `<path>`, the path in jig's `docs/design/` (jig's
  `domain/connectors.md`, jig's `client/domain/client.md`), never by a
  path on disk.
