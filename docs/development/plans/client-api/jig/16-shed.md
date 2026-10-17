# Session 16: the shapes of work leave

The reference UI's lists of chats and goals become task queries and its
chat and goal acts go through `jig_client::shapes`; then the shapes of
work leave jig: the 41 shape-specific records go from the wire (jig 13
removed the ten of the small store), with their translations, the
contract's types and the scripted engine's models of them; and the
engine's shape-specific code goes from the core, the people and tasks
children, the store and the configuration. `ops`'s wire crate is jig
21's, not this session's. Nothing named
for a chat, a goal or a list remains in the engine or the wire
(code-organization.md, 6).

## Read first

- jig's `docs/design/client/domain/pages.md`, sections 3, 4, 6 (6.2 and
  6.3) and 12.5; `client/domain/contract.md`, sections 1, 3.4, 3.6, 5.1,
  6.1 and 7.1; `client/domain/client.md`, section 19;
  `protocol/client.md`, section 3; `domain/engine.md`, sections 10 and
  11.4.
- The plan's `decisions.md`, 3; `findings/wire-and-client.md`, sections 1
  (the 51 records by name) and 2 (what goes from the page half);
  `findings/engine-parties-and-tasks.md`, section 2 (the shape-specific
  code to shed and its users); `findings/engine-agents-store-config.md`,
  section 3 (the configuration's `chats`).
- jig 14's, jig 22's and jig 15's session files: their interface sketches are the
  types this session moves the reference UI onto.
- skein's `programming-model.md`, sections 4.4 and 6.3;
  `testing-strategy.md`, sections 2.5, 4 and 7; skein's
  `docs/design/codec.md`, sections 3 and 6.

## What the design asks

- **The reference UI's lists are task queries** (pages.md, 12.5):
  conversations watch "`tasks{query}`: the party as requester, the chat
  template's label, the section's project if any, live, by last
  activity" and read "the same query, ended, by when they ended: the
  index of ended tasks by requester"; a project's goals watch "the
  project, tracked, live, by priority" and read "the project's ended
  tasks of the goal template's label, by when they ended: the index of
  ended tasks by project".
- **Its chat and goal acts go through `shapes`** (pages.md, 4 and 6.2):
  "Starting a chat: a project and the first words. `shapes::chat` fills
  the project's chat template into a `Create`"; "Closing is
  `shapes::close_chat`'s compound request"; "Moving a goal up or down is
  ... one compound request of priority-only amendments, one for each
  goal whose priority changes ... It names no revision"; setting a goal
  is `shapes::set_goal`, the form saying before it is sent whether the
  goal will be made, proposed and to whom, refused, or cannot be set.
- **No shape on the wire** (protocol/client.md, 3): "No record names a
  shape of work"; and none in the engine: "No operation, document or
  subject names a shape of work: a chat and a goal are tasks with
  fields" (contract.md, 1).

## Choices this plan makes

- **The chat template's label and the goal template's label are read
  from `me`** (`shapes::template(me, project, shapes::CHAT)`'s and
  `GOAL`'s `label`), never written as numbers in the reference UI; a
  project with no such template shows no conversations section, and no
  goals board, saying so.
- **Each of the 51 records goes, or is renamed in place to its general
  form** (the table in 16.4 and 16.5): `NoteScopeGoal` becomes
  `NoteScopeTask { project, task }`, the scope of a tracked task, which
  the notes child keeps under its own name (engine.md, 10 names "a goal"
  among the notes' scopes); `ReplyPosition` becomes `ReadPosition {
  task, through }`; `OwnerInboxObjectReply` becomes
  `OwnerInboxObjectTurn { message }`. `Write`, `MadeWords` and
  `SeenReplies` went in jig 14.
- **The lists leave with their engine code, and the asks with theirs:**
  each removal takes a record, its translation at both ends, its
  contract type, the scripted engine's model and the engine's code
  together, so that no exhaustive match maps a removed thing to
  nothing.
- **The scripted engine models the operations the reference UI now
  sends** (`Create` from a chat or a goal template, `Propose`, and the
  compounds closing a chat and moving goals) in place of `StartChat`,
  `SetGoal`, `Close` and `Prioritise`, giving the same outcomes its
  stories assert; this ends jig 14's "answered as not offered".

## Rules that bind it

- **No shape of work in the engine or the wire** (code-organization.md,
  6): the README's "Done when" 5 grep prints nothing after 16.5, and no
  operation, document, subject, record or index named for a chat, a goal
  or a list is added.
- **Deletion-heavy increments count added lines only** (the count of
  code-organization.md, 7, which counts added lines); a removed
  mechanism's tests are moved to its replacement or removed (README,
  Rules).
- **jig never names the repository downstream of jig:** its engine
  root's route of `people::Ask`, its store's sizing of `people::Ask` and
  its records of `people::Refusal::Disposal`, its family's refusal
  translation and its worlds' stories (23 files at the base, 3 of them
  production, by this session's third "Done when" grep run there)
  follow these removals at its repin; code-organization.md, 6's grep for that name prints nothing.

## Increments

### 16.1 The conversations section on a task query
Scope: the reference UI's conversations section (pages.md, 4 and 12.5):
its list watched as a task query and its ended chats read below it; each
line the chat's task line; the inbox's reply entries carrying the
conversation's task line (pages.md, 3).
Needs: the session's start: jig 21's reference UI with its state on the
client alone (after jig 13's reference UI on `jig_client`);
jig 22's task line, `tasks{query}` watched and read (22.2, 22.3) and
`me`'s templates (22.1); jig 15's `facts::chat_status` (15.1), `words`
(15.2) and `shapes::CHAT` and `template` (15.3): it starts once 15.1 to 15.3
have landed, beside 15.4 to 15.6 (ruling 248).
Files: the reference UI (`crates/jig-client-view/src/`): its
`Section::Conversations { project }` (at the base
`crates/jig-domain-client/src/pages/conversations.rs`, 515 lines, and
`crates/jig-view/src/sections/`), the inbox's reply rows; the scripted
engine's tasks subject and read (`testing/jig-client-scripted-engine/src/{watches.rs,
reads.rs}`: a query by requester and label, ended rows from its truth);
`tests/client`'s conversation stories; `tests/client-protocol`.
Shape: `jig_client_view`'s conversations section opens `Watch {
subject: Subject::Tasks { query: queries::conversations(party, label,
project), window } }` and reads `Read::Tasks { query: the same with
`phase: Some(PhaseClass::Ended)` and `order: QueryOrder::Ended`,
.. }` a page at a time, each time the person presses the section's
"Older" button; its line shows the chat's
title, project, `facts::chat_status` said by `words`, its unread turns
(`conversation.unread`), what it waits on (`waits_on`, said by
`words`), its live subtree counted (`phases`) and its spend (`budget`);
the inbox's reply entry
holds a `TaskLine` where it held a `ConversationLine`. The frame's
Conversations entry shows only while some project of `me` holds a
template of the chat template's number (`shapes::CHAT`), as pages.md, 2
has the entry for an application that offers chats. The query
functions are the interface sketch's.
Test shape: `tests/client/tests/stories.rs` (scripted):
`the_conversations_section_lists_live_chats_by_last_activity`,
`the_frame_shows_no_conversations_entry_without_a_chat_template` (a
`me` whose projects hold no template 1);
`tests/client-protocol/tests/protocol.rs` (joint), reference test (a new
kind: the reference UI's list through the fake person, over the real
engine's task query):

```rust
#[test]
fn a_person_s_chats_list_live_then_ended_by_last_activity() {
    use skein_fake_person::{Find, Step};
    let (mut story, client) = stories::ready(fixtures::SEED);
    stories::start_chat(&mut story, client, b"Second chat");
    stories::end_chat(&mut story, client, b"Hello");
    story.person(
        client,
        vec![
            Step::Go { address: stories::conversations() },
            Step::See { find: Find::text(b"Second chat"), within: skein_lib::Duration::from_secs(1) },
            Step::Press { find: Find::named(skein_ui::Role::Button, b"Older") },
            Step::See { find: Find::text(b"Hello"), within: skein_lib::Duration::from_secs(1) },
        ],
    );
    stories::finish(&mut story);
    assert_eq!(story.front().task_queries_read(), 1);
}
```

(`stories::ready` and `stories::finish` are at
`tests/client-protocol/src/stories.rs:17, 24` at the base, followed to
jig 12's file; `start_chat`, `end_chat`, `conversations` and the front's
`task_queries_read` counter are added there and in the front; the
button's name is the reference UI's own.) Ceilings: world under 0.5 s,
joint under 0.5 s.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_person_s_chats_list_live_then_ended_by_last_activity)'` once:
the live chat lists first, and the ended one appears below it after one
read of the ended group from the store's index. Bound: 1 s.
Don't: filter or order lines in the reference UI; write the chat label
as a number.
Size: at most 250 lines.
Removes: the reference UI's uses of `Subject::Conversations`,
`Read::Conversations`, `ConversationLine` and `ConversationPage` (the
records go in 16.4).

### 16.2 The goals board on a task query
Scope: a project's goals (pages.md, 6.2 and 12.5): the board watched as
a task query by priority, ended goals read below it; moving a goal as one
compound request of priority-only amendments; the project's overview
watching the goals' query for its first rows (pages.md, 12.5: "a
project's overview | `inbox`, filtered by the project; the goals' query,
its first rows"); and a card whose act is refused where the refusal says
a proposal could be accepted offers to propose it, with a confirmation
of its own (objects.md, 9; view.md, 18.4: "where the refusal says a
proposal could be accepted, offers to propose it, with a confirmation of
its own").
Needs: 16.1's query helpers in the reference UI.
Files: the reference UI's project goals section (at the base
`crates/jig-domain-client/src/pages/project/goals.rs`, 335 lines, and
`owner_goals.rs`, 77), its move-up and move-down acts, the project's
overview section, its refusal card (`RefusalAuthority { proposable }`,
which jig 14.5 leaves as it is) and its confirmation; the scripted
engine's compound of priority-only amendments (in place of its
`Prioritise`); `tests/client`'s goal stories; `tests/client-protocol`.
Shape: the section watches `queries::goals(project)` and reads
`queries::ended_goals(project, goal_label)`; a line shows the goal's
chip and phase in words, its progress (`phases`), its spend against its
budget, its deadline and what is held below; moving a goal sends `Ask::Compound
{ project, operations }` of `Operation::Amend { revision: None, change:
Amendment { tracked: Some(Tracking::Tracked { priority: p }), .. } }`
(ruling 260), one per goal whose priority
changes, and the board says what priority does (pages.md, 6.2). The
overview watches `queries::goals(project)` with a window of its first
rows, under its own handle. A refusal for authority whose `proposable`
is `Some(true)` offers "Propose instead", whose confirmation says to
whom from the offer (`me`'s creating bounds, or the task document's
amending bounds: `Beyond::Proposed { to }`; jig's contract.md, 4: "Who
would accept a proposal is the offer's to say") and sends
`Ask::Propose` of the refused act (a batch for a refused `Create`, an
amendment for a refused `Amend`); `Some(false)` says that nobody may
accept it, and `None` offers nothing.
Test shape: `tests/client/tests/stories.rs`:
`moving_a_goal_up_sends_one_compound_request_of_priorities`,
`the_project_overview_shows_the_board_s_first_goals`,
`a_refusal_a_proposal_could_answer_offers_to_propose_with_its_own_confirmation`,
`a_refusal_nobody_could_accept_offers_no_proposal`;
`tests/client-protocol/tests/protocol.rs`:
`moving_a_goal_up_reorders_the_board_in_one_compound_request` (two
goals the first owner set, by `shapes::set_goal`; the fake person moves
the second up; the engine admits one request; the board's next delivery
lists it first). Ceilings as 16.1.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(moving_a_goal_up_reorders_the_board_in_one_compound_request)'`
once: one keyed request is made (`requests_made()` 1), and the board's
first line becomes the moved goal. Bound: 1 s.
Don't: send one request per goal; name a revision in a priority-only
amendment.
Size: at most 300 lines.
Removes: the reference UI's uses of `Subject::Goals`, `Read::Goals`,
`GoalLine`, `GoalPage`, `Goals` and `Ask::Prioritise`.

### 16.3 The chat and goal acts through `shapes`
Scope: starting a chat, closing it and setting a goal in the reference
UI through `shapes` (pages.md, 4 and 6.2; client.md, 19); `me`'s
transitional `charters` and `chat_budget` go.
Needs: 16.2.
Files: the reference UI's start-a-chat form, close dialog and set-goal
form (at the base in `crates/jig-domain-client/src/{acts.rs, intents.rs,
pages/}` and `crates/jig-view/src/`); the v8 schema (`Me.charters`,
`MeProject.chat_budget`, `Charter`, `OfferedCharters` go); the
component's `translate/owner.rs`; the contract's `me.rs`; the link's
documents' translation; the scripted engine (`testing/jig-client-scripted-engine/src/{requests.rs,
payload.rs, truth.rs}`: `Create` from the chat and goal templates,
`Propose`, the compound closing a chat, each giving the outcome its
`StartChat`, `SetGoal` and `Close` gave); `tests/client-protocol`'s
front, whose chat counter counts tasks the engine made under the chat
label (it counted `StartChat`); `tests/client`'s stories.
Shape: the start form gives `shapes::chat(me, project, words)`; the
close dialog lists each live delegate with its fate and gives
`shapes::close_chat(me, project, chat, &delegates)`, each a
`shapes::Delegate { task, fate }`; the goal form
builds a `shapes::GoalDraft`, shows `shapes::fall` against the
project's `me` creating bounds (`MeProject.creating`) as it is written (made; proposed, and to
whom; refused, and why; cannot be set), and sends what
`shapes::set_goal` gives; a `NotOffered` sends nothing and says why.
Test shape: `tests/client/tests/stories.rs`: the existing
`signing_in_and_starting_a_chat` and
`a_goal_beyond_the_allotment_is_proposed_as_the_confirmation_said` keep
their names and expected values (the scripted engine now models
`Create` and `Propose`); add
`closing_a_chat_cancels_one_delegate_and_moves_the_other_to_the_person`
(view.md, 18.10: the chat has two live delegates; the close dialog
cancels one and moves the other; one compound request);
`tests/client-protocol/tests/protocol.rs`: the existing
`a_person_signs_in_by_the_printed_link_and_starts_a_chat` keeps its
assertions (`chats_made()` 1). Ceilings as 16.1.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(=a_person_signs_in_by_the_printed_link_and_starts_a_chat)'` once:
the person's chat is made by a `Create` from the chat template and the
real engine counts one chat. Bound: 1 s.
Don't: let the reference UI fill a template itself; keep a fallback to
`StartChat`, `Close` or `SetGoal`.
Size: at most 300 lines.
Removes: the reference UI's uses of `Ask::StartChat`, `Ask::Close`,
`Ask::SetGoal`, `Disposal`, `GoalExecutor`, `Bounds::Goal`; `Me.charters`,
`MeProject.chat_budget`, `Charter`, `OfferedCharters` on the wire and in
the contract.

### 16.4 The lists leave the wire and the engine
Scope: the conversation and goal lists' records, translations, contract
types and models, and the engine's list code (protocol/client.md, 3;
contract.md, 1; findings/engine-parties-and-tasks.md, 2).
Needs: 16.1 and 16.2 (nothing on the client reads a list record).
Files: the v8 schema; the component's `translate/{reads.rs, watches.rs,
pages.rs, pages/*.rs, inbox.rs}`; the contract and the link's
translations; the scripted engine; `testing/jig-client-test-records`;
`crates/jig-domain-core/src/{documents.rs (Subject, Watched),
documents/{project_tasks.rs, task_tree.rs, inbox_deployment.rs},
reads/{vocabulary.rs, cursors.rs, indexed.rs, rows.rs,
rows/positions.rs}, inbox.rs, routing/watch_marks.rs, indexes.rs,
forget.rs}` (`forget.rs` is jig 08.3's: its erase of tags 14 and 15
goes); `crates/jig-domain-tasks/src/{index.rs, closing.rs}` (the
Conversation and Goal keys and the `Goal` row's save);
`crates/jig-protocol-store/src/{keys/tasks.rs, codec/tasks/index.rs,
reads/indexed.rs, reads/joins.rs, intervals.rs}`; their tests
(`tests/core` reads and projections,
`crates/jig-domain-core/src/tests/conversation_cursor.rs`, the store's
`tests/reads/indexed.rs` and `tests/intervals.rs`). Where step 2 already
removed a piece, `rg -l -w -e ConversationNewest -e ConversationOldest -e
'Stored::Goal' -e 'Scope::Conversations' -e 'Scope::Goals' -e
'Watched::Conversations' -e 'Watched::Goals' -e 'Read::Conversations' -e
'Read::Goals' crates/jig-domain-core/src crates/jig-domain-tasks/src
crates/jig-protocol-store/src` at the increment's start lists no file
for it, and nothing is done for it. Shared hotspots:
`crates/jig-domain-core/src/{documents.rs, routing/mod.rs}`,
`crates/jig-protocol-store/src/keys/`.
Shape: the records that go, or are renamed in place:

| Record | Fate |
|---|---|
| `ReadConversations`, `ReadGoals`, `SubjectGoals`, `Read.conversations`, `Read.goals`, `Subject.conversations`, `Subject.goals` | gone: task queries |
| `ConversationLine`, `ConversationPage`, `Conversations`, `GoalLine`, `GoalPage`, `Goals` | gone: `TaskLine`, `TaskPage`, `Tasks` |
| `ReplyPosition { chat, through }` | renamed `ReadPosition { task, through }` |
| `OwnerInboxObjectReply { message }` | renamed `OwnerInboxObjectTurn { message }` |
| `Place`, `Draft`, `JigRecord`, `Address`, `AddressInbox`, `AddressConversations`, `AddressTask`, `AddressProject`, `ProjectSection`, `TaskSection` | gone in jig 13 (the reference UI's records leave the client's store); nothing here |

and in the engine: `documents::{Subject, Watched}::{Goals,
Conversations}` and `Tasks { project, filter }` where jig 05 left it,
their documents, `reads::Read::{Conversations, Goals}` and
`reads::Scope::{Conversations, Goals}` with `reads/cursors.rs`, the
conversations known by their charter (if jig 05 left it); the store's
tasks tags 14 (`ConversationNewest`), 15 (`ConversationOldest`) and 16
(`Goal`), as jig 08 left them for this increment (its file, 8.2 and
8.4): their keys, values and codecs, their saves (the core's
`indexes.rs` conversation rows, the tasks child's `Goal` row at a
goal's end), their readers (the conversation and goal parts of
`reads/rows/positions.rs` and of both `reads/indexed.rs`, the core's and
the store's) and the forget's erase of tags 14 and 15; tasks v3 and
people v3 edited in place. Tag 13, the unread results' index by
`(party, position)`, stays: it names no shape of work.
Test shape: tests of what goes are removed with it, or moved to the task
query's tests where they check a behaviour the query keeps (README,
Rules); no new test. Ceilings unchanged.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_person_s_chats_list_live_then_ended_by_last_activity) |
test(moving_a_goal_up_reorders_the_board_in_one_compound_request)'` once:
both lists still read through task queries with the list code gone.
Bound: 2 s. And `git grep -n -E
'^(record|enum) (ConversationLine|ConversationPage|Conversations|GoalLine|GoalPage|Goals|ReadConversations|ReadGoals|SubjectGoals|ReplyPosition|OwnerInboxObjectReply)\b'
-- crates/jig-client-protocol-wire/schema`, once in the worktree,
prints nothing.
Don't: keep a list record for another user; add an index.
Size: at most 120 lines added.
Removes: the table's records and the engine's list code above, tags
14 to 16 among it.

### 16.5 The shape asks leave the wire and the engine
Scope: the shape asks' records, translations, contract types and
models, and the engine's chat and goal asks, routes and settings
(findings/engine-parties-and-tasks.md, 2; engine-agents-store-config.md,
3).
Needs: 16.3 (nothing on the client sends a shape ask).
Files: the users are what this session's third "Done when" grep lists
at the increment's start; at the base it lists 207 files, 100 of them
production, by crate: the client crates (`jig-domain-client` 25 and
`jig-view` 12, now the contract, `jig-client` and the reference UI;
the link 8; the component 6: `translate/{asks.rs, answers.rs,
refusals.rs, owner.rs}`), the scripted engine 4, the test records 2;
`crates/jig-domain-core/src/` 21 (`acts.rs`, `acts/{bounds.rs,
objects.rs, objects/decisions.rs, objects/task.rs, requests.rs,
standing.rs}`, `chats.rs`, `goals.rs`, `documents.rs`,
`documents/{inbox_deployment.rs, offers.rs, task_tree.rs}`, `inbox.rs`,
`person_task.rs`, `reads/{indexed.rs, vocabulary.rs}`, `roles.rs`,
`routing/{parties.rs, party_controls.rs, people.rs}`; 16.4 takes the
lists' part); `crates/jig-domain-people/src/` 7 (`boundary.rs`,
`capacity.rs`, `domain.rs`, `lib.rs`, `limits.rs`, `refusals.rs`,
`words.rs`); `crates/jig-domain-tasks/src/` 4 (`boundary.rs`, `chat.rs`,
`domain.rs`, `lib.rs`); `crates/jig-protocol-store/src/codec/people/{asks.rs,
outcomes/facts.rs}`; the test kits that translate people refusals or
make chats: `testing/jig-fake-parties/src/lib.rs` (its `Ask::Close` and
chat asks, at the base lib.rs:54 and 396, become `Create` from the chat
template and the compound close), `testing/jig-test-connector/src/application_core.rs`,
`testing/jig-test-domain/src/{admission.rs, application_facts.rs}`,
`testing/jig-test-service/src/family/facts.rs`,
`testing/jig-test-store/src/records/facts.rs`,
`testing/jig-client-test-view/src/{connector_values.rs,
connector_values_refusal.rs}`; and `crates/jig-domain-core/src/lib.rs`
(`Settings.chat`, `close_grace`), `crates/jig-protocol-config/src/{lib.rs,
charters.rs, core.rs}` (what jig 09 left of `chats`),
`crates/jig-protocol-store/src/codec/tasks/proposals.rs` (`PersonProposal`,
if jig 03 left it); `crates/jig-protocol-config/src/templates.rs`
(jig 09.1's transitional `templates::chat` and the check it serves: a
template 1 naming no charter or no grace refused, `What::Refers` at
`templates`; its test moves or goes). Tests: `tests/core`'s `stories/closing_chat.rs` and
`procedure_goals.rs`, `tests/people`, `tests/engine-clients/tests/closing.rs`,
`crates/jig-domain-tasks/src/tests/revisions/chat.rs`, and the 107 test
files the grep lists. Shared hotspots:
`crates/jig-domain-core/src/{lib.rs, routing/mod.rs}`,
`crates/jig-domain-people/src/boundary.rs`, `crates/jig-protocol-store/src/codec/`.
Shape: the records that go, or are renamed in place:

| Record | Fate |
|---|---|
| `StartChat`, `Close`, `Disposal`, `DisposalCancel`, `DisposalTakeOver`, `SetGoal`, `GoalExecutor`, `GoalExecutorCharter`, `GoalExecutorProcedure`, `Prioritise`, `Priority` | gone: `Create`, `Propose` and `Compound`, through `shapes` |
| `NoteScopeGoal { project, goal }` | renamed `NoteScopeTask { project, task }` |
| `OwnerOutcomeClosing`, `OwnerOutcomeGoalStarted`, `OwnerOutcomeGoalProposed`, `OwnerOutcomePrioritised`, `MadeChat`, `MadeGoal` | gone: `OwnerOutcomeCompound`, `OwnerOutcomeCreated`, `OwnerOutcomeProposed` |
| `DisposalIssue`, `RefusalDisposal`, `LimitConversationWords` | gone: a compound's refusal at its place; `LimitWords` |
| `OwnerHistoryChangePrioritised { priority }` | gone: an amendment's history line |
| `BoundsGoal`, `GoalProcedure` | renamed `BoundsCreate`, `CreateProcedure` in jig 22.1; nothing here |
| `ActKind`'s `close`, `set_goal`, `prioritise` | gone: `cancel`, `move_task`, `amend` offered, and `create` bounded in `me` |
| `OwnerPersonCapacityAction`'s `close`, `chat`, `goal`, `priority` | gone with their asks (the people child's capacity actions of the operations are jig 03's and jig 04's) |

and in the engine: `people::Ask::{StartChat, SetGoal, Prioritise,
Close}`, `people::{Disposal, Keep, GoalExecutor}`,
`people::Outcome::{Closing, GoalStarted, GoalProposed, Prioritised,
Started}`, `people::Refusal::Disposal` and `DisposalIssue`, and each
match arm naming them; the core's chat and goal creation
(`routing/parties.rs:134-242` at the base), `chat_admit` and the
prioritising (`roles.rs:75-114, 228-272`), `chat_authority`
(`funding.rs:40-54`), the goal bounds (`acts/bounds.rs:22-91, 187-300`),
the goal decision (`routing/party_decisions.rs:16-97`), `chats.rs` and
`goals.rs` where jig 03 and jig 04 left them, `acts::Act::{Close,
SetGoal, Prioritise}`, the set-goal offer's judging (`acts::Bounds::Goal`
is renamed `Bounds::Create`, the creating bounds `me` carries, its
fields kept), the people child's capacity
actions of the removed asks, `Settings.chat` and `Settings.close_grace`
with their readers in `jig-domain-core` (9 at jig 09's count, its file,
"Transitional until jig 16") and jig 09.1's `templates::chat`, which
filled them, with its template-1 check; the tasks child's chat close and
`PersonProposal` where jig 03 left it; the store's codecs of these.
Test shape: removed tests' behaviours that the operations keep move to
them (closing a chat becomes `shapes::close_chat`'s compound in
`tests/core`'s stories, kept under their names and expected values);
others are removed. Ceilings unchanged.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(=a_person_signs_in_by_the_printed_link_and_starts_a_chat) |
test(closing_a_chat_by_shapes_moves_its_delegate_then_cancels_it_after_its_grace)'`
once: chats are made and closed through the operations with the shape
code gone. Bound: 2 s. And the README's "Done when" 5 grep, once in the
worktree, prints nothing.
Don't: keep a shape ask "for compatibility"; translate a removed record
into an operation on the engine's side.
Size: at most 150 lines added.
Removes: the table's records and the engine's shape asks above,
`Settings.chat`, `Settings.close_grace` and `templates::chat` among them.

## Interface sketches

Checked at the plan's base: the queries compiled in a scratch crate
with jig 22's sketch types (`heavy cargo clippy -p w7-sketch
--all-targets -- -D warnings` clean); checked again at the ready check.
The reference UI reaches the contract's types through `jig_client`'s
re-export of them.

```rust
// crates/jig-client-view/src/queries.rs: the reference UI's lists (pages.md, 12.5).
/// The conversations section's live query: the party's tasks of the chat template's label, by last activity.
pub const fn conversations(party: u64, chat_label: u32, project: Option<u32>) -> TaskQuery {
    TaskQuery {
        project,
        requester: Some(OwnerRequester::Person(party)),
        root: None,
        parent: None,
        label: Some(chat_label),
        phase: None,
        executor: None,
        tracked: None,
        order: QueryOrder::Activity,
    }
}
/// The goals section's live query: the project's tracked tasks, by priority.
pub const fn goals(project: u32) -> TaskQuery;
/// The goals section's ended read: the project's ended tasks of the goal template's label, by when they ended.
pub const fn ended_goals(project: u32, goal_label: u32) -> TaskQuery;
```


## Questions to raise when reached

- None the design leaves open: the lists' queries are pages.md, 12.5's
  table, and the forms' judgements contract.md, 5.3's.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep -c
  '^jig-16\.'` prints 5.
- The README's "Done when" 5 grep prints nothing.
- `git -C ~/src/rust/jig grep -l -E
  '\b(StartChat|SetGoal|Prioritise|GoalExecutor|Disposal|DisposalIssue|TakeOver|ConversationLine|GoalLine|OfferedCharters|BoundsGoal|GoalProcedure|ReplyPosition|MadeChat|MadeGoal|RefusalDisposal|LimitConversationWords|ReadConversations|ReadGoals|SubjectGoals|NoteScopeGoal)\b'
  origin/main -- '*.rs' ':!*/generated/*'` prints nothing (207 files at
  the base).
