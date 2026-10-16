# The engine's plumbing, parties' and tasks' half, at the base (step 2)

The code audit of jig at `a3b53546`, 2026-10-15, by a read-only
sub-agent, from source (the graph had no coverage for these crates).
Paths: C = `crates/jig-domain-core/src`, T = `crates/jig-domain-tasks/src`,
P = `crates/jig-domain-people/src`, A = `crates/jig-domain-authority/src`.
Estimates are hand-written production lines.

## 1. Each item

- **Create** (about 500: P 180, C 250, store codec 70).
  `T::Event::Make{creator: Party}` already takes person creators and
  executors (T/batch.rs:106, 138, 261). Parties create only through
  `begin_goal_creation` and `begin_chat_creation`
  (C/routing/parties.rs:134-242); no `P::Ask::Create`
  (P/boundary.rs:259-339). Missing: the ask, its validation (about 11
  match arms per new `Ask` variant), a core route for reach, person
  executors (reuse C/delegation.rs:165-181), resources, label, flags and
  tracked. Store: `people-v2.schema`, `codec/people/asks.rs` (668).
- **Propose and Withdraw** (about 550). `Proposal.proposer` is only a
  task (T/proposals.rs:132-140); a party proposes only a tracked root
  goal through `PersonProposal{goal: New}` (T/proposals.rs:143-178);
  `WithdrawProposal` is for tasks only (T/boundary.rs:1262). Missing: a
  party proposer on a batch, an amendment and a widening; the two asks
  and their outcomes; `goal_decide` generalized
  (C/routing/party_decisions.rs:16-97); "by whom" in refusals
  (P/refusals.rs:14-18 has `proposable: Option<bool>`). Store:
  `codec/tasks/proposals.rs` (664), `tasks-v2.schema`.
- **Explicit proposing** (about -150/+60 in C and P). Two implicit paths:
  a goal beyond the pool becomes `GoalProposed` (C/acts/bounds.rs:
  290-300); an amendment beyond the role has its Refuse rewritten to
  Propose (C/acts/bounds.rs:373-391, C/routing/party_controls.rs:62-86,
  `AmendProposed` at P/boundary.rs:468).
- **Cancel with a grace** (about 200, of which T about 120 moves from
  chat.rs). A grace exists only for closing a chat
  (`TaskRecord.chat_close`, T/boundary.rs:858; T/chat.rs:37-242), its
  span a deployment setting (C/lib.rs:120;
  `jig-protocol-config/src/charters.rs:22`). `P::Ask::Cancel`
  (P/boundary.rs:284) and `T::Control::Cancel` (T/control.rs:13-18) take
  no grace. Missing: a grace-span limit in tasks.
- **Move** (about 20). Exists (P/boundary.rs:294, `move_admit` at
  C/roles.rs:14-72, C/routing/party_controls.rs:369-397, T/moving.rs,
  432 lines). Missing: the escalation holder's check.
- **The compound request and operations per request** (about 900: T 350,
  C 300, P 180, configuration 20, codecs 50; the largest). Absent.
  Precedents: `Ask::Close` with T/chat.rs (checks every disposal before
  changing anything), `Say{release}` (P:298), `Amend{release}` (P:304),
  `Prioritise` (P:302, its per-request limit `goals` at P/limits.rs:29).
  Needs amend and release split into a check and an apply first
  (T/control.rs:192-410 and 540 onwards), and the configuration's check
  at start.
- **Label and flags** (about 150, and about 60 in
  `codec/tasks/record.rs`). Absent from `T::New` (T/boundary.rs:761-790)
  and `TaskRecord` (T/boundary.rs:844-990).
- **Last activity** (about 80). Absent; `TaskRecord` has `changed` (:864)
  and an activity position (:879). Set at T/admission.rs:42 (a turn),
  T/closing.rs:295 (a delegate's end), and on words admitted.
- **Waiting on** (about 250, a new core module and
  C/routing/watch_marks.rs). No symbol yet.
- **`AsAllowed`** (about 180: A 120, C 40, P 20). Absent from A; the
  chat's budget is resolved by hand in `chat_authority`
  (C/funding.rs:40-54) and not answered (`Outcome::Started{task}`,
  P/boundary.rs:491). Needs a budget form on `Delegate`
  (A/boundary.rs:131-138), resolved in A/check.rs.
- **Goals per project; tracked and priority amended by a maintainer
  only** (about 250). Tracking is set only at creation; `Amendment` has
  no tracked or priority (P/amendment.rs:10-20, T/control.rs:22-34);
  priority goes through `Prioritise` (T/control.rs:411-462,
  C/roles.rs:228-272).
- **A task's goal, the nearest tracked task at or above it** (about
  120). Three computations disagree
  (C/documents/project_tasks.rs:90-98, C/documents/task_tree.rs:653-667,
  C/routing/watch_marks.rs:193-197); projections are keyed by the tree's
  root (C/projections.rs:156).
- **The root's escalation holder** (about 80). Routing exists
  (C/escalation.rs:11-15, 70-120; `escalation_role` at A/rules.rs:34);
  the admission check is only in `chat_admit` (C/roles.rs:101-106), and
  wrong (section 3).
- **`me` and task queries** (about 650; jig-domain-views about 0, its
  subjects opaque). `me` is `Frame` (C/documents.rs:273-290, built in
  C/documents/small.rs:88-149), with `OfferedCharter{chat, goals}` and
  `chat_budget` (C/documents.rs:251-266). Lists are three shaped
  subjects, Goals, `Tasks{project, filter}` and Conversations
  (C/documents.rs:112-151; project_tasks.rs; task_tree.rs:305-318,
  876-959; inbox_deployment.rs:53-68, 321-356). Watch marking uses
  `ViewTask` (T/domain.rs:72-78).

## 2. The shape-specific code to shed (about 3,000 to 3,500 lines)

- **Core (about 1,800):** chats.rs 114; goals.rs 155;
  routing/parties.rs:134-242; roles.rs:75-114, 228-272;
  funding.rs:40-54; acts/bounds.rs:22-91, 187-300;
  routing/party_decisions.rs:16-97; inbox.rs:66-163, 359-450;
  inbox_deployment.rs's conversations (about 95); task_tree.rs's goals
  (about 100); project_tasks.rs 169; reads/cursors.rs 230;
  reads/indexed.rs:137-257; `Settings.chat` and `close_grace`.
- **People (about 400):** `Close`, `SetGoal`, `Prioritise`, `StartChat`,
  `GoalExecutor`, `Disposal`/`Keep` and six outcomes
  (P/boundary.rs:245-339, 422-495); chat read positions in inbox.rs
  (about 200 of 791); about 44 match arms in domain.rs.
- **Tasks (about 450):** chat.rs 242 (its grace half stays);
  `PersonProposal`; `prioritise`; the Conversation and Goal index keys
  (T/index.rs:53-58, 108-119, 193-197).
- **Its users in jig:** `jig-protocol-engine-clients` maps the wire one
  to one onto these asks (translate/asks.rs, 539 lines: 11, 15, 83, 87,
  148) and uses the subjects (owner.rs:161, pages.rs, reads.rs,
  watches.rs, answers.rs:63); `jig-protocol-store` encodes them
  (codec/people/asks.rs:350, 383, 459, 498; outcomes.rs;
  codec/tasks/proposals.rs); `jig-protocol-config` reads `engine.chats`
  (charters.rs; core.rs:55-58). Tests and kits:
  `testing/jig-fake-parties/src/lib.rs:54, 396`; tests/core (reads.rs,
  runs.rs, projections.rs, stories/procedure_goals.rs,
  src/stories/closing_chat.rs); tests/people;
  tests/engine-clients/tests/closing.rs; tests/ops's conformance;
  C/tests/conversation_cursor.rs (544); T/tests/revisions/chat.rs (161).
- **Downstream:** the downstream repository's engine root matches
  `people::Ask` (`StartChat`, `SetGoal`, `Prioritise`, `Say`, `Move`) in
  its `route/asks.rs`, and its store crate depends on jig-domain-people
  and core: each change of `Ask` reaches it at its repin.

## 3. Where the code differs from the design (beyond STATUS's paragraph)

- C/documents/task_tree.rs:653-667 overwrites the goal at each tracked
  parent, so it keeps the topmost tracked task, not the nearest
  (core.md, 6.2).
- C/roles.rs:101-106 admits a chat only when the policy names an
  escalation role numbered 3 or below; it ignores whether the creator's
  role decides escalations, refuses policy roles such as "the person on
  call", and answers `NotOffered` (people.md, 4 and 5.1; authority.md, 6
  and 8.4).
- A person task addressed to a policy role (numbered above 3) is refused
  on an agent's delegate (C/delegation.rs:168-173) and on restore
  (C/stored.rs:446) (tasks.md, 3; people.md, 8).
- Implicit proposing (C/acts/bounds.rs:290-300, 373-391) contradicts
  people.md, 5.1; tasks.md, 4 and 9; authority.md, 8.1 and 9;
  contract.md, 3.5.
- A conversation is known by its charter being `settings.chat`
  (C/inbox.rs:107-121; C/routing/watch_marks.rs:202-206;
  C/documents/inbox_deployment.rs:339-345), not by its flag (engine.md,
  11.4).
- No goals-per-project bound: tracked tasks count against the
  deployment's live tasks less projections (C/routing/tasks.rs:
  1351-1357) (tasks.md, 11).
- Projections are keyed by the root (C/projections.rs:156): a tracked
  task that is not a root gets none (core.md, 6.2).
- `move_admit` checks no escalation holder (tasks.md, 4 and 7).
- The chat's `AsAllowed` is resolved in core and not answered
  (authority.md, 7).
- The grace is a deployment setting, with no span limit (tasks.md, 7 and
  11).
- Priority changes are admitted for any policy role holding Amend
  (C/acts/standing.rs:14-17; C/roles.rs:235-240); people.md, 5.1 says a
  maintainer.
- Refusals never say who could accept a proposal (contract.md, 3.5).

## 4. Order

- Label, flags and last activity before Create, the task line and
  conversations known by their flag.
- `AsAllowed` and the escalation holder before Create; Create's batch
  before Propose; Propose before implicit proposing goes.
- The grace, tracked and priority amended, and goals per project before
  the compound request.
- Waiting on before task queries.
- The shed last: before step 3's wire it needs about 200 lines of
  temporary translation in `jig-protocol-engine-clients`, or it moves
  into step 3.
- The ended part of task queries and "forgotten once read" wait for the
  ended indexes and read positions (the store's half).

## 5. Worlds and test kits

- Create, Propose and Withdraw, explicit proposing, the escalation
  holder, `AsAllowed`: tests/tasks (src/world.rs 1,204,
  src/accounting_referee.rs 437, tests/lifecycle.rs 778,
  tests/person.rs 343); tests/people (src/world.rs 1,062,
  tests/simulation.rs 748); tests/core (tests/stories.rs 1,881,
  src/peers.rs 759, tests/referee.rs 732); testing/jig-fake-parties
  (lib.rs 448).
- The grace, the compound request, Move:
  tests/core/src/stories/closing_chat.rs 226;
  tests/engine-clients/tests/closing.rs 337; T/tests/revisions.rs 966.
- Fields, waiting on, queries, `me`: tests/core (tests/reads.rs 725,
  src/watches.rs 612, src/observations.rs 1,067, tests/activity.rs 286);
  tests/store and tests/store-codecs for the schemas.
- Goals: tests/core/tests/projections.rs 335; stories/procedure_goals.rs
  201; tests/tasks/tests/lifecycle/goals.rs 49.
- The largest unit test files: C/tests.rs 2,987, P/tests.rs 1,596,
  A/tests_policy.rs 1,345, C/tests/acts.rs 984.
