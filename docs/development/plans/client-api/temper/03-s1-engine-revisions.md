# Supplement temper-03.s1: temper's engine on jig 06's revisions and answers

Added by ruling 153, from temper 03's block of 2026-10-16T17:02:26Z.
temper 03.2 needs its landing. Read first: temper's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan" ("Pins", "Files over 800 lines"); `temper/03-client.md`, "Rules
that bind it".

Scope: the repin that 03.2's Needs name, onto jig 10.3's landing
`94eaa0340`, and with it what jig 06.1 (`350da6c54`) and 06.2
(`4b2ef1e69`) changed in the agents' tools, which no temper step
carries. jig's `domain/engine.md`, 7.3: "`amend`, `release` | a task
below it: an amendment naming its spec's revision as the run last read
it, but for a change of priority alone; a release naming its hold's,
and, for an effect held uncertain, whether to make it again | done,
with the task's spec revision after the commit, the one its next
amendment names (a release moves none); refused, and why; moved, with
what it is now"; "`decide` | ... | done; beyond its authority, and
passed up; moved, with what it is now". temper adapts them the way
those two commits adapted jig's own reference code
(`testing/jig-test-service/src/tools.rs`, `tests/core/src/agents_tools.rs`,
`crates/jig-protocol-tools/src/{declarations,decode,render}.rs`).
Needs: jig 10.3's landing (no temper step).
Files, at temper `0ff45022`:
- the repin commit, as the README's "Pins" says (03.2's, already made
  on `client-api/temper-03-2`, moves here), with the repin's other
  repairs within its 200 lines (an `EndedByParty` arm in
  `restart.rs:423`'s match among them);
- `crates/temper-engine-smith/src/calls.rs`: the `amend` schema (`:43`)
  gains a required `revision`, the `release` schema (`:46`) a required
  `revision` and an optional boolean `again`; their decoding (`:121`,
  `:140`);
- `crates/temper-engine-domain/src/boundary.rs`: `Tool::Amend { target,
  revision, amendment }` (`:140`), `Tool::Release { target, revision,
  again: Option<bool> }` (`:144`); `route/tools.rs`: `Amend` (`:157`) to
  `NamedAction::Amend { target, revision, amendment }`, `Release`
  (`:171`, a `Control::Release` today) to `NamedAction::Release {
  target, revision, again }` (jig `vocabulary.rs:601-610` at
  `94eaa0340`); `translate.rs`'s arms (`:58`, `:84`, `:135`, `:238`,
  `:297`) follow;
- `CallAnswer` (`crates/temper-engine-domain/src/store.rs:19`) gains
  `Again { task, required }`, `PassedUp { task }`, `Revised { task,
  revision }`, `Moved { task, now }`, mapped both ways in `core_part`
  and `from_core` (jig `stored.rs:22-28`), with any field jig 06 added
  to a part temper maps; `route.rs:320`'s and `forge_route.rs:2349`'s
  (`effect_answer`) matches take them as core-owned; `temper-engine-smith/src/answers.rs`
  (`answer`, `:96`) renders each to the run, naming its task and value;
- the tests' literals: `tests/engine/domain/tests/root_routes.rs:1893,
  :1928, :1947, :2024`, `crates/temper-service/src/tests/tools.rs:29`;
  a further user the compiler names gets the same, listed in the
  commit body.
Test shape:
- world, in `tests/engine/domain/tests/escalation.rs` (400 lines):
  `an_amend_naming_an_old_spec_revision_is_answered_moved` (a delegate
  amended once is amended again naming its first revision: answered
  moved, with its present revision; nothing committed);
  `an_amends_answer_names_the_revision_its_next_amendment_names`;
  `a_release_naming_an_old_hold_revision_is_answered_moved`;
- unit, in `crates/temper-engine-smith/src/tests.rs`:
  `amend_and_release_calls_name_their_revision` (each without
  `revision` refused as malformed, with it decoded);
  `each_of_jigs_new_call_answers_renders_its_task_and_value` (the four).
Proof: world, once: `heavy cargo nextest run -p
temper-engine-domain-world --test escalation -E
'test(an_amend_naming_an_old_spec_revision_is_answered_moved)'`;
temper's gate, by `land`.
Size: aim for at most 220 production lines besides the repin's
repairs; past it, report why. The repin's 200 lines stay a rule.
Don't: change temper's store format (`store-v1.schema:120`'s
`CallPayload` holds the connector's answers alone; jig's core v3 keeps
these four); unify or rename temper's tools (`decide` and
`decide_escalation` stay two); give `ProposedAction`'s `Amend` or
`Release` a revision (jig's `proposals.rs:88, :92` have none); touch
temper's client crates (03.2's).
Done when: `git -C ~/src/rust/temper log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: temper-03.s1$' --format=%H
-1` prints its landing.
