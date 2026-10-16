# Session 23: the conformance referee's rules for proposals and revisions

Two of the referee's promises that jig's design names (`domain/testing.md`,
section 6) and that no rule of jig's conformance referee checks at the
base. The first: nothing waits as a proposal that its proposer did not
ask for. The second: no amendment or release is made against a revision
that moved after the one its party, run or procedure named. Each is a rule
of `jig_conformance::referee`, fed by observations that each adapter
supplies through the harness, with a violation test per rule. The rules
check sessions jig 03 (proposing made explicit), 06 (the agents' tools
naming revisions) and 07 (the procedures' revisions) from outside. Nothing
here changes the engine.

## Read first

- jig's design: `domain/testing.md`, sections 5 (the conformance world:
  "a party's and a run's decisions about one task racing") and 6 (the
  referee: "Authority holds: ... nothing waits as a proposal that its
  proposer did not ask for"; "Nothing written over: ... no amendment or
  release made against a revision that had moved since the one its party,
  run or procedure named"); `domain/tasks.md`, sections 7 ("A decision
  names what it saw") and 9 ("Proposing is asked for"); `domain/people.md`,
  5.1 ("A decision names what it saw"; "A hold's revision moves with the
  hold").
- The plan: `README.md` (Rules), `code-organization.md` (6, 7); session
  files `jig/03-parties.md` (3.4, 3.5), `jig/06-agents-tools.md` (6.1, 6.2)
  and `jig/07-connectors.md` (7.3), for the names this session observes.
- skein's `testing-strategy.md`, sections 6 and 7.

## What the design asks

- **No proposal unasked** (testing.md, 6, "Authority holds"): "nothing
  waits as a proposal that its proposer did not ask for: a run's
  `propose`, a step's proposal, a party's `Propose`, or a result's
  follow-ups." jig 03.4 and 03.5 remove the engine's two paths that made
  proposals unasked; this rule keeps them gone.
- **Nothing written over** (testing.md, 6): "no amendment or release made
  against a revision that had moved since the one its party, run or
  procedure named." jig 06.1's `Scenario::RacingDecisions` races a
  party's and a run's amendments; this rule judges each shared story, not only
  that one.
- **The referee watches from outside** (testing.md, 6): "what the fakes
  saw, what the store made durable, and the facts the domain emits, never
  a domain's state." The rules read only `Observed` values and durable
  snapshots.

## Choices this plan makes

- **Asks and named revisions are observations**, which each adapter
  derives from the root's inbound events (`Application::inbound`,
  `testing/jig-conformance/src/harness.rs:138`) and returns from
  `Application::observed` (:158): a party's `Propose`, a run's `propose`
  call or `propose_batch`, a run's terminal with follow-ups; a party's
  `Amend`, `Release` and `DecideEscalation`, a run's `amend`, `release`
  and `decide`, each with the revision it names.
- **A proposal is matched to an ask by its proposer, not by its number**:
  the number is the core's, given at the commit. An ask answered refused
  takes back its credit (`Observed::ProposeRefused`), so a later unasked
  proposal by the same proposer is caught.
- **A procedure's proposals and amendments**: a procedure decides inside
  the application's root, out of the adapter's sight. Its proposals are
  counted as asked when the proposer task's executor is a procedure,
  since a procedure proposes only in its step (connectors.md, 6.3). Its
  amendments' named revisions come from the test connector's durable
  `ProcedureState` (jig 07.3), which the testing application's adapter
  reads from the store.
- **The spec revision moves by one per amendment** (`revisions.rs` at the
  base): a rise from `r` to `r + k` between two durable snapshots needs
  `k` named amendments naming `r` to `r + k - 1`. A task leaving
  `Phase::Held` for `Waiting` or `Active` needs a named release or
  escalation decision naming the hold revision it had; leaving it for
  `Closing` or `Ended` (a cancel) needs none.

## Rules that bind it

- **`testing/jig-conformance/src/referee.rs` is 1,108 lines**: each
  increment adds at most 40 lines to it (code-organization.md, 7, `git
  diff --numstat origin/main...HEAD -- <file>`, first column), its rule in
  a new module beside it, as `referee/conversations.rs` and
  `referee/application.rs` are. `tests/core/tests/referee.rs` is 732
  lines: the new violation tests go in modules of their own under
  `tests/core/tests/referee/`.
- **Shared files** (README, Rules): `testing/jig-conformance/src/` with
  jig 04.7, 06.1 and 08.3 to 08.6; `tests/core/src/observations.rs`,
  `tests/conformance/src/lib.rs` and `tests/ops/src/conformance/observations.rs`
  (the adapters) with the same sessions.
- **The referee names no domain type** (`referee.rs:2-5`): the new
  observations carry numbers and the referee's own `Source`; each adapter
  translates. jig's files never name the repository downstream of jig;
  its conformance adapter is the application's (testing.md, 5, "What the
  application gives"), and its repin absorbs the new observations.

## Increments

### 23.1 No proposal waits that its proposer did not ask for
Scope: testing.md, 6, "Authority holds": a durable proposal of a task or
a party is matched to an ask of its proposer, or the referee answers
`Promise::Authority`.
Needs: jig 03.5's explicit proposing (`people::Ask::Propose`; neither
`GoalProposed` nor `AmendProposed` left in the engine): at the session's
start.
Files:
- `testing/jig-conformance/src/referee.rs` (at most 40: `Observed::{ProposeAsked,
  ProposeRefused}`, `Snapshot.party_proposals`, the dispatch in `observe`,
  :855 on), `testing/jig-conformance/src/referee/proposals.rs` (new: the
  rule); shared with jig 04.7, 06.1, 08.3 to 08.6.
- The adapters: `tests/core/src/observations.rs` (the core world:
  `party_proposals` from `tasks::Stored::PersonProposal` rows; the asks
  from its inbound `people::Event::Ask` and `core::Event::NamedAction`
  and its workers' terminals), `tests/conformance/src/lib.rs` (the testing
  application), `tests/ops/src/conformance/observations.rs` (`ops`), each
  building `Snapshot` with the new field.
- `tests/core/tests/referee.rs` (`proposals_referee_cases()` called by
  `referee_scenario_family`, :712), `tests/core/tests/referee/proposals.rs`
  (new, `#[path]`-included as `tests/core/tests/stories.rs:11` includes
  `stories/address.rs`).
Shape:
- `Observed::ProposeAsked { proposer: Source }` and
  `Observed::ProposeRefused { proposer: Source }` (sketch); `Snapshot`
  gains `party_proposals: BTreeMap<u64, u64>`, each pending party
  proposal's number and its proposing party.
- The rule, on each `Observed::Durable`: each proposal number first seen
  in the snapshot (a `Task.proposal`'s, proposer `Source::Task(<that
  task>)`, or a `party_proposals` entry, proposer `Source::Person`)
  consumes one credit of its proposer; none left is
  `Violation { promise: Authority, why: "proposal <n> of <proposer> waits
  unasked" }`. A task proposal whose proposer's executor is a procedure is
  counted as asked (Choices). `ProposeAsked` adds a credit,
  `ProposeRefused` takes one back. Credits and seen numbers survive
  `Observed::Restart`, as the referee's other evidence does.
Test shape:
- `tests/core/tests/referee/proposals.rs`, with the file's `Fake`
  (`tests/core/tests/referee.rs:117`), `task` (:51) and `snapshot()`
  (:94): `authority_catches_a_proposal_its_proposer_did_not_ask_for`,
  `authority_catches_a_proposal_after_its_ask_was_refused`,
  `a_proposal_asked_by_its_party_is_valid`,
  `a_proposal_asked_by_its_run_is_valid`,
  `a_procedures_proposal_needs_no_observed_ask`. Each under 20 ms.
  Reference:

  ```rust
  fn authority_catches_a_proposal_its_proposer_did_not_ask_for() {
      let mut fake = Fake::ready(Recovery::Keyed);
      let mut rows = snapshot();
      rows.party_proposals.insert(40, 1);
      fake.caught(Observed::Durable { number: 2, snapshot: rows }, Promise::Authority);
  }
  ```
Proof: the conformance world, `heavy cargo nextest run -p
jig-conformance-world -E
'test(every_shared_adversarial_story_runs_on_the_testing_application)'`
once (`tests/conformance/tests/scenarios.rs:10`): each shared story,
proposals among them, runs on the testing application's real root over
the real core and store with the rule on, and none is caught; it passes,
and nothing else is wanted; under 0.5 s. This is the highest layer for a
referee rule: the referee is the conformance world's. The violation tests
show the rule can fail.
Don't: read a domain's state in the referee; change the engine, a
scenario's script or an existing rule.
Size: at most 40 lines (`testing/jig-conformance` only; the adapters
under `tests/` are not counted, code-organization.md, 7).
Removes: nothing.

### 23.2 No amendment or release is made past the revision it named
Scope: testing.md, 6, "Nothing written over": each move of a task's spec
revision, and each release of a hold, is matched to a named request with
the revision it moved from, or the referee answers `Promise::Ownership`.
Needs: jig 06.1's `NamedAction::Amend { revision }` and
`Scenario::RacingDecisions`, jig 06.2's `NamedAction::Release { revision
}`, jig 07.3's `StepDecision::Amend { revision }` and the test
connector's `ProcedureState { delegate, revision }`.
Files:
- `testing/jig-conformance/src/referee.rs` (at most 40: `Revision`,
  `Observed::Named`, `Task.spec_revision`, `Task.hold_revision`, the
  dispatch), `testing/jig-conformance/src/referee/revisions.rs` (new: the
  rule); shared as 23.1's.
- The adapters, as 23.1's: `Task`'s two revisions from the task row's
  `revisions` (`TaskRecord.revisions.spec`, `.hold`), `Named` from their
  inbound asks and named actions; the testing application's adapter also
  from the test connector's durable `ProcedureState` (Choices).
- The users of `referee::Task`'s literal, edited to the new form:
  `tests/core/tests/referee.rs:51` (`task`) and the three adapters.
- `tests/core/tests/referee/revisions.rs` (new), and
  `revisions_referee_cases()` in `tests/core/tests/referee.rs`.
Shape:
- `referee::Revision { Spec(u64), Hold(u64) }`; `Observed::Named { task:
  u64, revision: Revision }`; `Task` gains `spec_revision: u64` and
  `hold_revision: u64` (sketch).
- The rule, on each `Observed::Durable`, for each task in both the last
  snapshot and this one: each `r` in `last.spec_revision..now.spec_revision`
  consumes one `Named { task, Spec(r) }`; a task leaving `Phase::Held`
  for `Waiting` or `Active` consumes one `Named { task, Hold(<last
  hold_revision>) }`; none left is `Violation { promise: Ownership, why:
  "task <n>'s spec moved past revision <r>, which nothing named" }` (or
  its hold). A named request that never commits leaves an unused entry,
  which nothing reads.
Test shape:
- `tests/core/tests/referee/revisions.rs`, the same fixtures:
  `ownership_catches_an_amendment_made_past_the_revision_it_named`,
  `ownership_catches_a_release_of_a_hold_nobody_named`,
  `two_amendments_each_naming_the_revision_they_moved_are_valid`,
  `a_release_naming_the_hold_after_a_pass_up_is_valid`,
  `a_cancel_of_a_held_task_needs_no_named_hold`. Each under 20 ms.
  Reference:

  ```rust
  fn ownership_catches_an_amendment_made_past_the_revision_it_named() {
      let mut fake = Fake::ready(Recovery::Keyed);
      fake.send(Observed::Named { task: 1, revision: Revision::Spec(0) }).expect("a named amendment");
      let mut rows = snapshot();
      rows.tasks.get_mut(&1).expect("task").spec_revision = 1;
      fake.commit(2, rows.clone()).expect("the amendment naming revision 0 is made");
      rows.tasks.get_mut(&1).expect("task").spec_revision = 2;
      fake.caught(Observed::Durable { number: 3, snapshot: rows }, Promise::Ownership);
  }
  ```
Proof: the conformance world, `heavy cargo nextest run -p
jig-conformance-world -E
'test(every_shared_adversarial_story_runs_on_the_testing_application)'`
once: the shared stories, `RacingDecisions` among them, run with the rule
on and none is caught. It passes, and nothing else is wanted. Under 0.5 s,
at the layer and for the reason 23.1 gives.
Don't: count an accepted amendment proposal as unnamed (its proposer
named the revision when it proposed, and that `Named` is consumed at the
acceptance); read a task's revisions anywhere but the durable snapshot.
Size: at most 60 lines (`testing/jig-conformance` only).
Removes: nothing.

## Interface sketches

`testing/jig-conformance/src/referee.rs`, the additions (compiled at
jig's base `a3b53546` as a scratch module of `jig-conformance`, under
`heavy cargo clippy -p jig-conformance --all-targets -- -D warnings`:
passed). The variants and fields join the existing `Observed`, `Snapshot`
and `Task`.

```rust
/// Which revision a decision names (domain/people.md, 5.1; jig 23.2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Revision {
    /// A task's spec revision, named by an amendment.
    Spec(u64),
    /// A task's hold revision, named by a release or an escalation's decision.
    Hold(u64),
}
// Observed gains:
//     /// A party's `Propose`, a run's `propose` call, or a result's follow-ups asked for a
//     /// proposal (domain/testing.md, 6; jig 23.1).
//     ProposeAsked { proposer: Source },
//     /// That ask was answered refused: nothing of it waits (jig 23.1).
//     ProposeRefused { proposer: Source },
//     /// A party, a run or a procedure named the revision its amendment or release decides (jig 23.2).
//     Named { task: u64, revision: Revision },
// Snapshot gains:
//     /// Each pending party proposal's number and its proposing party (jig 23.1).
//     pub party_proposals: BTreeMap<u64, u64>,
// Task gains (jig 23.2):
//     /// The task's spec revision, from its durable row.
//     pub spec_revision: u64,
//     /// The task's hold revision, from its durable row.
//     pub hold_revision: u64,
```

## Questions to raise when reached

- **23.2: a hold that lifts on its own.** If a story shows a task leaving
  `Phase::Held` for `Waiting` or `Active` with no party, run or procedure
  naming its hold (a hold whose reason the engine clears itself), the
  rule exempts that reason, and the increment names it in its report.
- **23.2: a procedure's named revision.** If the test connector's durable
  `ProcedureState` does not say which step amended, so that the adapter
  cannot report each amendment once, the increment raises it before it
  adds a field to that state.

## Done when

- Each step has landed: `git -C ~/src/rust/jig fetch origin && git -C
  ~/src/rust/jig log origin/main --all-match --grep='^Plan: client-api$'
  --grep='^Step: jig-23.1$' --format=%H -1` prints a commit, and the
  same with `--grep='^Step: jig-23.2$'` prints a commit.
- `git -C <a jig worktree at origin/main> grep -n -E
  'ProposeAsked|Observed::Named' -- testing/jig-conformance/src` prints
  lines from `referee.rs` and from `referee/proposals.rs` or
  `referee/revisions.rs`.
