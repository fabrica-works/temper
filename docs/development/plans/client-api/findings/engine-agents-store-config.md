# The engine's plumbing, agents', connectors', store's and configuration's half, at the base (step 2)

The code audit of jig at `a3b53546`, 2026-10-15, by a read-only
sub-agent, from grep and reading (the graph was 53 commits behind).
Estimates are hand-written production lines.

## 1. Each item

- **Agents' `amend` with a revision** (about 160): the tool takes no
  revision (`jig-protocol-tools` declarations.rs:22, decode.rs:43-46);
  `NamedAction::Amend` (`jig-domain-core` routing/vocabulary.rs:586)
  reaches runs.rs:489-541 unchecked, where the parties' path checks it
  (party_controls.rs:37 → revisions.rs:51). Missing: the revision, a
  "moved" answer, "done, with the revision made" (`CallPart::Controlled`,
  stored.rs:40, has no revision); the brief shows no revisions
  (brief_text.rs:294-315; tasks `DelegateState`, boundary.rs:1768).
  Changes protocol-tools, domain-core, domain-tasks, protocol-store
  (codec/core/calls.rs: core v3).
- **`release` with a revision** (about 110): decode.rs:33-36 always sends
  `Release{again:None}`; runs.rs:479-488 skips `hold_revision` (the
  parties' at party_controls.rs:251-276). Missing: the revision and
  "again"; a revision on `NoticeState::Held` (tasks boundary.rs:60);
  relayed escalations with their revision (jig-test-domain link.rs:
  347-381, jig-test-service tools.rs:343).
- **`withdraw`** (about 40): built (decode.rs:41, runs.rs:545-556); "already
  decided, and how" is missing (runs.rs:546 answers Unknown).
- **Cancel with a grace, the tool's side** (about 15): the tool takes only
  a reason (decode.rs:29-32; tasks `Control::Cancel{reason}`,
  control.rs:15).
- **Topics offered to agents** (about 250): pieces exist, used only by
  `ops`'s root (examples/ops/domain/src/route.rs:1106-1145):
  `SubscriptionKind::Topic` (tasks boundary.rs:71),
  `Event::SubscribeTopic` (domain.rs:551),
  `reserve_connector_subscription` (core routing/mod.rs:79),
  `SubscriptionDone`/`EndTopic` (vocabulary.rs:59, 85), `connector_news`
  (connector.rs:439). Missing: a topic kind in `subscribe`
  (decode.rs:73-90), the connector's offered topics and the resources
  each concerns, the read-grant check, connector news (dropped at
  effects.rs:400-405), the reference root's routes (routes.rs:751, 834).
- **Procedures' amend, cancel and message** (about 350, two increments):
  `ProcedureAction` (delegation.rs:39-44) is Delegate, Result, Hold or
  Wait; `Ask::ProcedureMade/Activate/Message/Close` are declared
  (connector.rs:263-275) and never sent; `ProcedureDelegateOutcome{child}`
  (vocabulary.rs:277) carries no revisions or reason. Changes
  domain-core (delegation.rs, run_delegation.rs), jig-test-connector
  (procedures.rs, `StepDecision`), jig-test-domain (routes.rs:621-660).
- **The party's own scope, and windows** (about 120): `ApplicationScope`
  is Project or Deployment (documents.rs:170; jig-test-connector
  client_documents.rs:7); `Ask::Document/Read` carry no window
  (vocabulary.rs:37, 39; views.rs:35; reads.rs:262-268).
- **Untracking** (about 50): no untrack transition in projections.rs
  (Absent/Open/Changed/Closing); tracked cannot be amended (tasks
  control.rs:21-33).
- **The two ended indexes** (about 250 added, about 600 removed): neither
  exists; in their place Result (tag 13), ConversationNewest/Oldest
  (14/15), Goal (16), EndedDelegate (17) (store keys/tasks.rs:57-86;
  tasks index.rs:63-125). The core saves the result and conversation
  rows (indexes.rs:6-21), the tasks child Goal and EndedDelegate
  (closing.rs:338, 379). Changes tasks (index.rs, closing.rs,
  boundary.rs:1040-1050), the store (keys/tasks.rs, codec/tasks/index.rs,
  tasks v3, intervals.rs, reads/indexed.rs, historical.rs, restore.rs),
  core (indexes.rs, reads/; cursors.rs 230, positions.rs, indexed.rs and
  the census go).
- **Ended read positions** (about 200): missing; people's
  `Key::ReadChat{party,task}` is one family, all restored at start
  (people boundary.rs:539-541; `live()` always true, :551-555). Changes
  people, the store (keys/people.rs, codec/people.rs, people v3, the
  restore range at intervals.rs:14, reads/joins.rs), core's census
  (restart/census.rs, inbox/census.rs).
- **Task queries' reads** (about 250): read scopes are Conversations,
  Goals and EndedDelegates (reads/vocabulary.rs:61-90); filtering past an
  index's prefix and continuing from the last row examined exist
  (rows.rs:432). Changes core reads/ and the store (intervals.rs:74-130,
  reads/indexed.rs).
- **Minted credentials' digests:** built (secrets.rs:43-49; kept with
  their expiry by engine-clients component.rs:495-501). `Mint` is step
  5's.
- **Templates and policies' overrides** (about 330): none; a `chats`
  member instead; neither the people policy (policy_value.rs:27-34) nor
  the authority policy (rules.rs:30-45) holds overrides. Changes config
  (a new templates.rs, projects.rs, core.rs), people
  (`PolicyValue`/`PolicyChange`), the store (codec/people/policy.rs,
  people v3), core (`Settings`, lib.rs:118-131).
- **Operations per request** (about 40): no people limit, no tasks
  "goals per project" limit. Changes people limits.rs, config (the check
  and profiles), core routing/mod.rs:275-294.
- **`clients`** (about 30): `Clients{listener,heartbeat,link}` (config
  clients.rs:6-27), no bundle member; the listener read at
  jig-test-service clients.rs:69 and jig-test tests/support/mod.rs:60.
- **`providers`:** built as kinds (config domain.rs:14-75).
- `jig-domain-host`, `jig-domain-fleet`, `jig-domain-notes`: no change.

## 2. The store's format

- The format row, format.rs:7: component 1, core 2, tasks 2, people 2,
  notes 1; schemas `schema/*-v2`; drift tests at
  tests/store-codecs/tests/drift.rs:42-57. Tasks tags 0-5 are live (the
  restore range ends at tag 6, intervals.rs:15); tags in use 0-18.
- The ended indexes add two tasks keys, `(party, label, ended_at
  reversed, task)` and `(project, label, ended_at reversed, task)`, on
  new history tags, each with a small row in tasks' `Value`
  (tasks-v2.schema:294): tasks v3; tags 13-16 retire.
- Ended read positions add people tag 7, `(task, party)`: people v3; the
  people restore range (`[JIG,2]`) stops before tag 7. The tools'
  answers change `CallPart`: core v3.
- Precedent: jig-protocol edited tasks v1 in place across commits, then
  bumped to v2 once (`c67c8f68`, 1,719 files): one bump per family per
  plan.
- Tests: in the store crate tests/reads/indexed.rs:117-250,
  tests/intervals.rs:131-230, tests/restore.rs:153 and 240,
  tests/keys.rs, the goldens; in tasks src/tests/index.rs:43, 50; the
  core world on the real store (tests/core/tests/reads.rs:286, 308, 616,
  649); tests/store-codecs (drift, fuzzy_decode, memory, profiles). The
  store's own world (tests/store) writes only live task records
  (root.rs:237): it covers no index.

## 3. The configuration

- `chats` is parsed at config lib.rs:93-98, then charters.rs:13-33
  (`Chats{charter, close_grace}`, :9-12), then core.rs:55, 58 into
  `Settings.close_grace` and `Settings.chat` (domain-core lib.rs:120,
  126). `templates` replaces it: the chat template carries the charter
  and the grace.
- Its users: config lib.rs:7, 50, 135, fixtures.rs:58, charters.rs:30,
  domain.rs:72-73; JSON fixtures (jig-test-service tests/fixtures.rs:49,
  tests/client-protocol native/config.rs:43, tests/channel
  fixtures.rs:78); `Settings` literals (domain-core tests.rs:263,
  jig-test-domain tests/fixtures.rs:118, tests/core world.rs:302,
  tests/engine-clients fixtures/receiving.rs:34); readers of
  `Settings.chat` (inbox.rs:112 `chat_requester` with 18 call sites,
  charters.rs:56, roles.rs:91, routing/parties.rs:229,
  documents/small.rs:128, documents/inbox_deployment.rs:342,
  routing/watch_marks.rs:205); readers of `close_grace` (chats.rs:93,
  acts/objects/task.rs:168).
- The worst case at start: operations at least max(delegates + 1, goals
  per project): 8 in the small profile (delegates 4, project_tasks 8), 9
  to 32 in the standard one; priced as operations × (tree_tasks +
  connectors + 4): 304 (small), 4,352 (standard).
  `route_room_checked` already adds tasks × (tree_tasks + connectors +
  4), so the check at start reduces to "operations ≤ tasks".

## 4. Where the code differs from the design (beyond STATUS's paragraph)

- A run's `release` of an uncertain effect is always refused
  (decode.rs:35 sends no "again"; tasks control.rs:557-558): hosts.md, 8.
- Escalations and held notices reach a run without their hold's revision
  (tasks boundary.rs:60; link.rs:365-369; tools.rs:343), so
  `decide_escalation` names a revision the run never saw: hosts.md, 9;
  engine.md, 7.3.
- The brief's delegates have no revisions (brief_text.rs:294-315); the
  Lineage, Calls and Waiting sections are never planned
  (run_preparation.rs:267-347; kinds at planned.rs:17-27): engine.md, 9.
- Separate `decide` and `decide_escalation` tools (declarations.rs:18,
  23), and an `introduce` tool (:17) not in engine.md, 7.3's table.
- `propose` decodes only "batch" (decode.rs:94-100), though the tasks
  child has Amend and Widen proposals (proposals.rs:88-90); `delegate`
  answers numbers only (stored.rs:58; render.rs:95-108): hosts.md, 8;
  engine.md, 7.3.
- The reference root panics on a procedure step that delegates or
  proposes (routes.rs:646); the four procedure asks are never sent:
  connectors.md, 6.2 and 6.3.
- Connector news is dropped in the core (effects.rs:400-405) and the
  reference root (routes.rs:751); topic asks are no-ops or unreachable
  there (routes.rs:264-267, 834); `ops` subscribes from its root with no
  authority check: connectors.md, 5.3; engine.md, 4.4.
- Application watches and reads carry no window to the connector
  (vocabulary.rs:37, 39): connectors.md, 5.5.
- The core, not the tasks child, saves the result and conversation index
  rows (indexes.rs:6-21): store.md, 7.4.
- The restart census reads each ended-conversation row (intervals.rs:28;
  restart/census.rs:18) and hands up their ended tasks (restore.rs:153):
  store.md, 7.2 and 15.2.
- Chat read positions are live at start, ended ones included, bounded by
  people × inbox_tasks (people limits.rs:66): engine.md, 6 and 13.
- `clients.listener` is still jig's (configuration.md, 3).

## 5. Order

- From the parties' and tasks' half: the label and flags (the indexes,
  ended positions, task queries, templates); the grace and its span
  limit (the tool's and procedures' cancels, the templates' grace);
  tracked made amendable (untracking); `AsAllowed` (`delegate`'s
  answer); parties' `Withdraw` ("already decided"); goals per project
  and the compound request (the operations' check); the task queries'
  keys, order and last activity (their reads); `me` (templates served);
  authority.md, 8.5 (the topic's read grant).
- Within this half: agents' revisions before procedures naming
  revisions; the ended indexes before task queries' reads; the people v3
  bump shared by ended positions and the templates' overrides.
