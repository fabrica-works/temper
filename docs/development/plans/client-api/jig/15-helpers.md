# Session 15: `jig-client`'s helpers

`jig-client` gains its helpers as modules, sugar over the public client
(client.md, 19): `facts`, pure functions over the documents the client
holds (a chat's status, what a decision will do, whether an act is
dangerous); `words`, jig's English for its enums as skein-ui phrases an
application overrides; `shapes`, plumbing requests filled from the
templates `me` serves (a chat, closing a chat, a goal); and `thread`, a
conversation's thread model, moved out of the reference UI's
conversation module onto the client's surface. The reference UI takes
`facts`, `words` and `thread` here; it moves its chat and goal forms
onto `shapes` in jig 16. Last, jig's check (testing.md, 6.2), which
holds an application's worlds to deciding on what was delivered and
seen, goes into `jig-client-test-kit`, over `facts`, and the reference
UI's worlds run it; the test application then presents a proposal
itself (15.6), held to the check, and the referee keeps offers.

## Read first

- jig's `docs/design/client/domain/client.md`, sections 4, 10 and 19;
  `client/domain/testing.md`, section 6 (6.2 for 15.5);
  `client/domain/contract.md`, 3.4, 3.6, 5.1, 5.3 and 5.4;
  `client/domain/pages.md`, section 4 (its status table) and 6.2;
  `client/domain/objects.md`, sections 1, 2 and 4 to 7;
  `client/domain/view.md`, section 7.
- The plan's `decisions.md`, 3; `findings/wire-and-client.md`, sections
  2 (the conversation module, 1,566 lines, "becomes the `thread`
  helper") and 5l.
- jig 14's and jig 22's session files (`jig/14-wire.md`,
  `jig/22-documents.md`): their interface sketches are
  the types these helpers read and fill.
- skein's `programming-model.md`, sections 3, 6.2 and 10;
  `testing-strategy.md`, sections 2.1, 2.5 and 7.

## What the design asks

- **Sugar over the public client** (client.md, 19): "jig's helpers are
  modules of `jig-client`, built only from plumbing (the API's
  operations, subjects and reads, contract.md) and from the client's
  public surface, as an application's own code is. A helper an
  application ignores costs it nothing."
- **`shapes`:** "`shapes::chat(me, project, words)` fills the chat
  template into a `Create`; `shapes::close_chat(task, disposals)` gives
  one compound request: each live delegate cancelled or moved to the
  party, then the chat cancelled with the template's grace;
  `shapes::set_goal(..)` fills the goal template into a `Create`, or a
  `Propose` where the project's offer says the creator's authority does
  not cover it." The comparison with the offer is contract.md, 5.3's.
- **`facts`:** "pure functions over the documents the client holds: a
  chat's status, from its phase, its run and what it waits on; what a
  decision will do, read from its document." pages.md, 4 gives the
  status table; objects.md, 2 adds "whether the act is dangerous".
- **`thread`:** client.md, 10: a model the application holds, fed the
  activity's pages, the task's document, its run's text and the task's
  pending `Message` requests with their answers; committed, read from
  the end; turns that do not decode kept as lines; live then committed;
  a person's words being sent, sent, read; bounded.
- **`words`:** "jig's English for its enums (phases, holds, refusals,
  what a task waits on), as phrases with holes an application overrides
  phrase by phrase; the reference UI says them through skein-ui's
  phrasebook (view.md, section 7)."

## Choices this plan makes

- **`facts` reads the contract's types through `jig_client`'s re-export**
  of them, as an application does; it takes the reader's `me` to tell a
  role the reader holds.
- **A chat's status takes what a task line carries** (its chip, what it
  waits on, its project), so that a list row and a chat's page say one
  status. "Waiting for them, by name" gets a person's number from
  `WaitsOn`; the name is the presentation's to find (neither the task
  line nor the task document carries it).
- **jig's check lives in `jig-client-test-kit`** (testing.md, 6.2), the
  renamed `jig-test-client-kit`, ordinary Rust; the client world's
  referee feeds it what the client delivered and what the person did;
  it supersedes skein-ui's `jig_view::check` (skein-ui's `later.md`).
- **The templates `shapes` fills are jig's numbers,** `shapes::CHAT = 1`
  and `shapes::GOAL = 2` (contract.md, 3.6's table); a project whose
  merged templates hold no template of that number gives
  `NotOffered::Template`, and the client sends nothing. An application
  numbering its shapes otherwise writes its own helper.
- **`close_chat` takes `me` and the project** besides the chat and its
  delegates' fates, to find the chat template's grace; the delegates'
  fates are `shapes::Delegate { task, fate: Fate::{Cancel, Move} }`
  (not `Disposal`, a name jig 16 removes from jig).
- **`set_goal` judges the draft** against the project's
  creating bounds in `me` (`MeProject.creating`, jig 22.1: `Creating {
  within, beyond }`) by `shapes::fall` (the interface sketch's
  algorithm), and answers `Judged`; a project whose `creating` is none
  gives `NotOffered::Creating`; a `Create`, a `Propose` and to whom, refused in
  place, or not offered.
- **`words` uses skein-ui's phrase types** (`skein_ui::phrases`), so
  `jig-client` depends on `skein-ui` for them; its phrases are owned by
  `words::OWNER`, `skein_ui::Owner { number: 65_537 }`, beside the
  reference UI's 65,536. It covers jig's `Phase`, `HoldCategory`,
  `Refusal`, `WaitsOn` and `facts::ChatStatus`.
- **`thread` moves the conversation model,** not its page routing: the
  model's files move into `jig_client::thread` in a commit of its own (a
  pure move, counted 0 by `-M`), with the window types it needs (`Edge`, `Edges`, `End`,
  `TurnWindow`), and its inputs become the public surface's (a request
  key where it held a request token); the reference UI keeps its routing,
  its chips and the naming of objects in its working set, over
  `Thread::lines`.

## Rules that bind it

- **Helpers are built from the public surface alone:** the files
  `crates/jig-client/src/{facts,words,shapes,thread}*` reach the
  contract's types through `jig_client`'s own re-export of them, as an
  application does, and name no `jig_client_domain`,
  `jig_client_protocol_link` or `jig_client_view` item (`rg -n
  'jig_client_domain::|jig_client_protocol_link::|jig_client_view' crates/jig-client/src/{facts,words,shapes,thread}*`
  prints nothing at each increment's end).
- **The meeting point:** each helper runs its Proof against the real engine in
  `tests/client-protocol`, beside its unit tests (README, Rules).
- **jig never names the repository downstream of jig** (code-organization.md, 6's grep for that name prints
  nothing).

## Increments

### 15.1 `facts`
Scope: `jig_client::facts` (client.md, 19; pages.md, 4's table;
contract.md, 5.3 and 5.4; objects.md, 2); the reference UI's
conversation page says the chat's status from it, and its confirmation
and cards read an act's effect and danger from it.
Needs: the session's start: jig 22's `TaskLine`, the task's `waits_on`
and fields (22.2), `me` (22.1) and `Offered.effect` (22.4); jig 12's
`jig_client::Client`.
Files: `crates/jig-client/src/{lib.rs, facts.rs}` (new module), its
tests `crates/jig-client/src/tests/facts.rs` and fixtures
`crates/jig-client/src/tests/fixtures.rs` (jig 12's); users: the
reference UI's conversation page (`crates/jig-client-view/src/`, the
conversation's header) and its confirmation (where it reads an effect
or marks a cancel or a rejection dangerous).
Shape: `pub enum ChatStatus { Starting, Writing, Tool { tool:
Option<Text>, since: Wall }, WaitingForYou { kind: WaitedFor, entries: u32 },
WaitingFor(Whom), Parked, Retrying { at: Wall, try_number: u32, tries: u32, class:
FailureClass }, Held(HoldCategory), Closing, Ended { at: Wall } }`;
`pub fn chat_status(chip: &Chip, waits_on: &WaitsOn, project: u32, me:
&Me) -> ChatStatus`, called with a task line's `chip`, `waits_on` and
`project` (a list's row) or a task document's (a chat's page) alike: the
chip's details carry the run's stage, its observed work (writing, or a
tool and since when), its retry notice, its hold and its end; waiting
for you when `waits_on` names the reader, or a role they hold in the
task's project, as `me` says; `pub fn effect(offers: &[Offered], act: ActKind) ->
Option<&Effect>`; `pub fn offer(offers: &[Offered], act: ActKind) ->
Option<&Admission>`; `pub fn dangerous(act: ActKind) -> bool` (cancel
and reject). The status follows pages.md, 4's table row by row: "due,
preparing or claimed | starting; running, writing | writing; running a
tool | which tool, when its host names it, and for how long; waiting on
the person ... | waiting for you, and for what; waiting on another party
| waiting for them, by name; idle, parked | parked; backing off |
retrying, when, and why; held | why, with its card; closing, then ended
| closing; ended, and when".
Test shape: `crates/jig-client/src/tests/facts.rs`, a table test
`chat_status_follows_the_pages_status_table` (a `const` of rows beside
it, one per row of pages.md, 4's table: a chip and a `WaitsOn` built by
the fixtures, the reader's `me`, the expected `ChatStatus` as a
literal), `a_task_line_and_its_task_document_give_one_status`,
`effect_is_none_for_an_act_not_offered`,
`dangerous_marks_cancel_and_reject_alone`; `tests/client-protocol/tests/client.rs` (jig 12's story file):
`a_running_chat_reads_as_writing_from_the_engine_s_documents` (jig 12's
`Story` with jig 14.2's `Story::running_task`: while the fake worker's
turn is in progress, the status over the task document and over the
task's line is `Writing`; once the fake worker's question commits, both
are `WaitingForYou { kind: WaitedFor::Question, entries: 1 }`). This
increment adds `Story::me` and `Story::line` to jig 12's `Story`
(`tests/client-protocol/src/client.rs`): the client's `me` handle, and
a task query's line for one task, watched under a handle of the
story's. Ceilings: unit under 20 ms, joint under 0.5 s.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_running_chat_reads_as_writing_from_the_engine_s_documents)'`
once: `facts::chat_status` over the client's latest task document, and
over its line in a task query's watch, gives `Writing` while the fake
worker writes, and
`WaitingForYou { kind: WaitedFor::Question, entries: 1 }` once its
question commits. Bound: 1 s.
Don't: say a status in words here (15.2); read anything but the values
the caller passes; take the run's document (the chip carries what the
status needs).
Size: at most 200 lines.
Removes: the reference UI's own reading of a cancel's effect where it
read the legacy `Impact` (jig 22.4 left the data in `Offered.effect`).

### 15.2 `words`
Scope: `jig_client::words` (client.md, 19; view.md, 7): a phrase for
each variant of jig's `Phase`, `HoldCategory`, `Refusal`, `WaitsOn`
and `facts::ChatStatus`, with jig's English; the reference UI loads them
beside its own phrases and says these enums through them.
Needs: 15.1's `ChatStatus`.
Files: `crates/jig-client/{Cargo.toml (skein-ui), src/words.rs (new),
src/words/english.rs (new)}`; the reference UI's phrasebook
(`crates/jig-client-view/src/phrases/{jig.rs, english.rs}`: their
variants that say these enums go, at most 40 added lines in `jig.rs`,
which is 3,046 lines at the base), its `Words` (loads
`words::phrase_shapes` and `words::english` beside its own, says a
`Word`), the places that
said those enums (at the base `crates/jig-view/src/{chip_phase.rs,
runtime.rs, owner_chip_words.rs, words.rs}` and the refusal words);
`tests/boundary`'s graph test if it pins `jig-client`'s dependencies.
Shape: as the interface sketch: `pub const OWNER: skein_ui::Owner`;
`pub enum Word { Phase(Phase), Hold(HoldWord), Refusal(RefusalWord),
Waiting(WaitingWord), Status(StatusWord) }`, each `*Word` an enum of
its type's variants without their fields; `impl Word { pub fn id(self)
-> PhraseId; pub fn hold(&HoldCategory) -> Self; pub fn
refusal(&Refusal) -> Self; pub fn waiting(&WaitsOn) -> Self; pub fn
status(&ChatStatus) -> Self }`; `pub fn phrase_shapes(text: u32) ->
Box<[PhraseShape]>`;
`pub fn english() -> Language`. Each mapping is an exhaustive match, so
a new variant is a build error until it is said.
Test shape: `crates/jig-client/src/tests/words.rs`:
`english_loads_beside_the_reference_ui_s_phrases_in_one_phrasebook`
(skein-ui's `Phrasebook::load` with both, capacity from
`jig_client_view`'s limits), `each_hold_category_says_its_own_phrase`
(a table over `HoldCategory`'s variants, expected English as literals),
`an_application_s_language_overrides_one_word_and_keeps_the_others`
(client.md, 19: "phrases with holes an application overrides phrase by
phrase": a language giving `Word::Phase(Phase::Held)` alone says its
own words for it and jig's English for the rest);
`tests/client-protocol/tests/client.rs` (jig 12's story file, against
the real engine): `a_stopped_task_s_hold_reads_in_jig_s_words` (jig
14.2's `Story::running_task`, stopped; `words` says the task
document's hold, `Word::hold`, as "stopped by you" in jig's English, a
literal). Ceilings: unit under 20 ms, joint under 0.5 s.
Proof: world and joint, `heavy cargo nextest run -p jig-client-world -E
'test(=a_task_page_with_its_conversation_prints_as_its_golden_text)'`
and `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_stopped_task_s_hold_reads_in_jig_s_words)'`, each once: the
reference UI's text shell prints the task page's golden unchanged, its
phases and holds now said by `words`; and the real engine's hold reads
in `words`' English. Bound: 1 s each. The
golden is the check that nothing a person reads changed; it is not
regenerated.
Don't: put the reference UI's sentences ("funds 40 from your pool")
into `words`; translate a name.
Size: at most 280 lines.
Removes: the reference UI's phrases for these enums (in `phrases/jig.rs`
and `phrases/english.rs`).

### 15.3 `shapes`
Scope: `jig_client::shapes` (client.md, 19; contract.md, 3.4, 3.6 and
5.3).
Needs: the session's start: jig 14's `Create` (14.4), `Propose` (14.5),
`Compound` with `Cancel`'s grace and `Move` (14.2, 14.3), `me`'s
templates and creating bounds (jig 22.1).
Files: `crates/jig-client/src/{lib.rs, shapes.rs}`, tests
`crates/jig-client/src/tests/shapes.rs`; `tests/client-protocol/tests/client.rs` (jig 12's story file)
and its stories module. No reference UI user until jig 16.3.
Shape: as the interface sketch: `pub const CHAT: u32 = 1; pub const
GOAL: u32 = 2;` `pub enum NotOffered { Project, Template, Executor,
Ceiling, Creating }`; `pub enum Fate { Cancel, Move }`; `pub struct Delegate {
task, fate }`; `pub struct GoalDraft { words, executor, budget,
deadline, priority }`; `pub enum Judged { Create(Ask), Propose { ask,
to: Recipient }, Refused(Refusal), NotOffered(NotOffered) }`; `pub enum
Fall { Within, Proposable(Recipient), Refused(Refusal),
NotOffered(NotOffered) }`; `pub fn chat(me: &Me, project: u32, words:
Text) -> Result<Ask, NotOffered>`; `pub fn close_chat(me: &Me, project:
u32, chat: u64, delegates: &[Delegate]) -> Result<Ask, NotOffered>`;
`pub fn set_goal(me: &Me, project: u32, draft: GoalDraft, reason: Text)
-> Judged`; `pub fn template(me: &Me, project: u32, number: u32) ->
Option<&Template>`; `pub fn fall(creating: &Creating, draft:
&GoalDraft) -> Fall`. `chat` fills a one-task `Create`: executor the
template's charter, budget `AsAllowed` with the template's cap, the
template's contract, flags, label and authority narrowing, no deadline
unless the template gives a span; `close_chat` a `Compound` of each
delegate's `Cancel` (no grace) or `Move`, then the chat's `Cancel` with
the template's grace; `set_goal` a `Create` within the first bound, a
`Propose` of a one-task batch between the bounds where proposable, with
the template's tracked and the draft's priority.
Test shape: `crates/jig-client/src/tests/shapes.rs` (a `me` built by
the fixtures with project 1's templates 1 and 2):
`chat_fills_the_chat_template_into_one_as_allowed_task`,
`close_chat_cancels_the_chat_last_with_its_template_s_grace`,
`fall_puts_a_draft_without_a_deadline_beyond_a_bound_that_has_one`,
`fall_gives_not_offered_for_an_executor_the_bounds_do_not_list`
(pages.md, 6.2: "with an executor not offered, it says the goal cannot
be set, and sends nothing"),
`set_goal_proposes_to_the_holder_the_bounds_name`;
`tests/client-protocol/tests/client.rs` (jig 12's story file), reference test (a helper's
request sent through the client and made by the real engine):

```rust
#[test]
fn a_chat_made_by_shapes_is_a_conversation_under_the_chat_label() {
    let mut story = Story::live(fixtures::SEED);
    let me = story.me();
    let ask = jig_client::shapes::chat(&me, 1, b"Hello".as_slice().into()).expect("project 1 offers chats");
    story.send(CHAT, Ask::Jig(Box::new(ask)));
    assert!(story.run_until(|s| !s.answered(CHAT).is_empty(), Duration::from_secs(1)));
    let [Outcome::Answered(Answered::Jig(contract::Answer::Done(contract::Outcome::Owner(
        contract::OwnerOutcome::Created { tasks, given },
    ))))] = story.answered(CHAT).as_slice() else {
        panic!("the chat is made");
    };
    assert_eq!(tasks.len(), 1);
    assert!(given.is_some());
    let line = story.line(tasks[0]);
    assert_eq!(line.label, 1);
    assert!(line.flags.conversation);
}
```

(`Story::me` and `Story::line` are 15.1's; `CHAT` is a `Key` constant
beside jig 12's `STOP`.)
Also `closing_a_chat_by_shapes_moves_its_delegate_then_cancels_it_after_its_grace`,
`a_goal_past_the_first_bound_is_proposed_to_the_policy_s_role`. Limits:
the configuration's templates from jig 09's default configuration, the
core's from `jig_core_world::world::limits()`. Ceilings: unit under
20 ms, joint under 0.5 s.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_chat_made_by_shapes_is_a_conversation_under_the_chat_label) |
test(closing_a_chat_by_shapes_moves_its_delegate_then_cancels_it_after_its_grace)'`
once: the engine makes the chat with label 1 and the conversation flag,
answering the budget it gave; closing it moves its delegate to the
person and cancels the chat once its grace has passed, in one request.
Bound: 2 s.
Don't: check authority or compute `AsAllowed` in `shapes`; send
anything from a helper (it returns requests; the caller sends them).
Size: at most 230 lines.
Removes: nothing.

### 15.4 `thread`
Scope: `jig_client::thread` (client.md, 10): the conversation model,
moved from the reference UI's conversation module onto the client's
surface. Its list of live turns stays: the thread keeps each turn's
provisional text until its committed turn, from the one turn the
client's `text(handle)` gives at each `Text` (client.md, 8.3).
Needs: the session's start: jig 14's `Message` (14.3) and its outcome;
jig 12's request keys and `client.pending(key)`.
Files: `crates/jig-client/src/{lib.rs, thread.rs, thread/window.rs,
thread/words.rs}`: moved from the conversation model (at the base
`crates/jig-domain-client/src/conversation.rs`, 791 lines, and
`conversation/words.rs`, 238 lines; where jig 11 to 13 put them, their
file names it) and the window types (`Edge`, `Edges`, `End`,
`TurnWindow`, at the base `crates/jig-domain-client/src/windows.rs:13-30,
335-390` and `action.rs`'s `End`); users: the reference UI's
conversation routing, chips, sending and shown modules (at the base
`conversation/{routing,chips,sending,shown}.rs`), its pages that hold a
conversation, its `windows.rs` (the moved types leave it), and
`name_objects`, which stays in the reference UI as a function over
`Thread::lines`.
Shape: as the interface sketch: `pub struct Limits { lines: u32, bytes:
u64, text: u32 }`; `pub enum WordsState { BeingSent, Sent { message:
u64 }, Read }`; `pub enum Line<'a> { Words { by, words, state }, Turn {
name, turn }, Undecoded { name, why }, Provisional { name, text, cut },
RunBegan { attempt }, RunEnded { attempt }, Left { lines } }`; `pub
struct Thread` with `new(task, &Limits)`, `document(&mut self, &Task) ->
Option<Read>`, `page(&mut self, ActivityPage) -> Option<Read>`,
`older(&mut self) -> Option<Read>`, `run(&mut self, &RunState,
Option<&RunText>)`, `sending(&mut self, key: Key, by: u64, words: Text)
-> bool`, `answered(&mut self, key: Key, &Answer)`, `lines(&self)`.
`Conversation` becomes `Thread` and `KeptWords` its words, in place;
request tokens become `jig_client` keys.
Test shape: the moved model's tests move with it (at the base the
tests of `crates/jig-domain-client/src/tests/conversations.rs` that
drive `Conversation` and `KeptWords` directly, their names and expected
values kept); `crates/jig-client/src/tests/thread.rs`
adds `a_committed_turn_replaces_its_provisional_text`,
`a_turn_that_does_not_decode_stays_a_line_saying_why`;
`tests/client-protocol/tests/client.rs` (jig 12's story file):
`a_person_s_words_read_as_being_sent_then_sent_then_read` (the thread
fed from the client's handles while the real engine admits the
`Message` and the fake worker's turn reads through it). Ceilings: unit
under 20 ms, joint under 0.5 s.
Proof: joint, `heavy cargo nextest run -p jig-client-protocol-world -E
'test(a_person_s_words_read_as_being_sent_then_sent_then_read) |
test(=a_person_s_words_reach_a_running_run_through_bytes)'` once: the
words' line moves from being sent to sent with the engine's message
number, then to read when the committed turn names it; the reference UI
still shows the words it sent. Bound: 2 s.
Don't: let `thread` read or send anything; keep a second copy of the
window types in the reference UI.
Size: at most 250 lines, the move aside: the two files and the window
types move first, in a commit of their own with no other change
(`git mv` and a cut, so `-M` counts them 0), and the edits follow in the
increment's next commits; code-organization.md, 7 lists jig 15.4 among
the moves.
Removes: the reference UI's conversation model and its window types
(moved, not copied).

### 15.5 jig's check in `jig-client-test-kit`
Scope: jig's check (`client/domain/testing.md`, 6.2): "Decided on what
was delivered: every decision that names a revision named one in a
document the client delivered to the application, for that object,
before the decision was sent"; and "A decision was seen, on a tree:
every deciding act a person made was made on a tree that showed its
decision's facts (what `facts` gives from the object's document ...),
in the open confirmation or within the acted node's nearest named
region". The reference UI's worlds (`tests/client`, the scripted world
and its joint world) run it in their referee; "checked, never mandated".
Needs: 15.1's `facts::effect` and `facts::dangerous`, 15.2's `words`
(the facts as the tree says them); jig 22's offers and effects (22.4).
Files: `testing/jig-client-test-kit/{Cargo.toml, src/lib.rs, src/check.rs}`
(the crate is 10 lines at the base, its `connector_values` kept; it
gains `jig-client` and `skein-ui`); `tests/client/src/harness/{referee.rs,
observation.rs}` (326 and 122 lines at the base): the referee feeds the
check each document the client delivered by handle, each keyed request
sent, and each tree a person acted on with the acted node; the world's
`Cargo.toml`.
Shape: as the interface sketch: `pub struct Check`; `pub enum Object {
Task(u64), Proposal(ProposalKey), Escalation(u64), PersonTask(u64), Note
{ project: u32, note: u64 } }`; `pub enum Fault { Undelivered { key:
Key, object: Object, revision: Revision }, Unseen { key: Key, act:
ActKind } }`; `impl Check { pub fn new(limits: &Limits) -> Self; pub fn
delivered(&mut self, document: Delivered<'_>); pub fn sent(&mut self,
key: Key, ask: &Ask) -> Option<Fault>; pub fn acted(&mut self, key: Key,
act: ActKind, tree: &Tree, node: NodeId, facts: &[&[u8]]) -> Option<Fault>
}`; `pub fn named(ask: &Ask) -> Option<(Object, Revision)>` (which
revision a decision names: a release's and an escalation's hold, an
amendment's spec, an answer's person task, a note's; contract.md, 5.2).
`acted` looks for each fact's words in the open dialog's text, or in
the text of the acted node's nearest ancestor with a region name. The
check keeps, per object, the revisions delivered, bounded by `Limits`.
Test shape: `testing/jig-client-test-kit/tests/check.rs` (the crate's
own tests, as `tests/connector_values.rs` is):
`a_decision_naming_a_delivered_revision_passes`,
`a_decision_naming_a_revision_never_delivered_is_a_fault`,
`a_revision_delivered_after_the_decision_is_still_a_fault`,
`a_cancel_acted_on_without_its_effect_shown_is_a_fault`,
`a_cancel_whose_effect_the_confirmation_shows_passes`; and in
`tests/client/tests/referee.rs`, beside its referee tests (`:274`, `:324`
at the base), reference test (a new kind: a fault the check catches in
a world):

```rust
#[test]
fn jig_s_check_fails_a_release_naming_a_revision_never_delivered() {
    let mut w = stories::held_chat(11);
    let c = w.client(7, stories::chat());
    w.front().send_raw(c, RELEASE, release(HELD_TASK, Revision(99)));
    let outcome = w.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Failed);
    assert!(matches!(
        outcome.faults.as_slice(),
        [Fault::Check(jig_client_test_kit::check::Fault::Undelivered { revision: Revision(99), .. })]
    ));
}
```

(`stories::held_chat` and `stories::chat` are `tests/client/src/stories.rs`'s;
`send_raw`, a request the scripted application sends past the
reference UI, `release`, `HELD_TASK` and `Fault::Check` are added to the
world.) Limits: the check's own, from the world's (`tests/client/src/test.rs:18`).
Ceilings: unit under 20 ms, world under 0.5 s.
Proof: world and joint, `heavy cargo nextest run -p jig-client-world -E
'test(=a_held_chat_is_released) |
test(=a_proposal_accepted_in_the_inbox_leaves_its_chat_too) |
test(jig_s_check_fails_a_release_naming_a_revision_never_delivered)'`
once: the reference UI's stories pass with the check in the referee,
and a made-up revision fails the world naming the check's fault; and,
in the joint world (the same crate's `joint`, the real core), `heavy
cargo nextest run -p jig-client-world --profile fuzzy -E
'test(=random_people_with_held_commits_and_double_press)'` once: its
random people decide on real documents with the check in the referee,
and it passes. Bound: 2 s, and 5 s for the fuzzy story.
Don't: mandate the check at run; read a client's state (the referee
watches what the client reported, as testing.md, 6 says).
Size: at most 300 lines.
Removes: nothing (skein-ui's `jig_view::check` was never built).

### 15.6 An application's own presentation, and offers kept
Scope: `client/domain/testing.md`, 4: the test application's "page also
presents one of jig's objects itself, a proposal ... saying what `facts`
gives, with a confirmation of its own", so that jig's check (15.5) meets
an application's own presentation and not only jig's card; and 6.3,
"Offers kept": "a draft within the bounds its act was offered with is
made, and one beyond them is proposed or refused as the offer said,
unless a newer document of the object shows its facts moved in between
(contract.md, 5.3)", as a rule of the client world's referee.
Needs: 15.5.
Files: `testing/jig-client-test-view/src/family.rs` (at the base
`testing/jig-test-view/src/family.rs:32-60`: its page slots jig's card
for each proposal; it gains its own presentation of the item's first
proposal: what it would do, from `facts::effect`, said by `words`, its
accept and reject acts behind a confirmation of its own, in a named
region); `testing/jig-client-test-domain` where the page's acts become
`Decide` requests; `tests/client/src/harness/{referee.rs,
observation.rs}` (a draft's offer and its answer observed: `Seen::Draft {
key, offer: Fall }` beside the sent request, the answer's outcome);
`tests/client/tests/{referee.rs, stories.rs}`.
Shape: the family page's own proposal section names its region (the
check's "nearest named region") and shows each fact `facts::effect`
gives; its confirmation is its own (a dialog of the test view's, not
`jig_client_view`'s). The referee's rule: for each draft the person
sent, the request's answer matches the offer the person saw, `Within`
made, `Proposable(to)` proposed to `to`, `Refused` nothing sent, unless
a newer document of the object, delivered before the request left,
moved the offer's facts.
Test shape: `tests/client/tests/stories.rs`:
`the_test_application_s_own_proposal_view_is_held_to_jig_s_check`
(the person accepts the family page's own proposal after its
confirmation shows the funding; the check passes), and
`an_own_proposal_view_hiding_its_effect_fails_jig_s_check` (a variant
of the test view that leaves the effect out: the referee fails it with
`Fault::Unseen`); `tests/client/tests/referee.rs`:
`a_draft_within_its_offer_answered_proposed_fails_offers_kept`,
`a_goal_past_the_first_bound_answered_proposed_keeps_the_offer`.
Ceilings: world under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-client-world -E
'test(the_test_application_s_own_proposal_view_is_held_to_jig_s_check) |
test(=a_goal_beyond_the_allotment_is_proposed_as_the_confirmation_said)'`
once: the application's own presentation passes jig's check, and the
existing goal story passes with "offers kept" in the referee. Bound:
2 s. A joint layer adds nothing here: the presentation and the referee
are the world's, and 15.5's joint story runs the referee over the real
core.
Don't: let the test view's own presentation reuse jig's card; mandate
either rule at run.
Size: at most 250 lines.
Removes: nothing.

## Interface sketches

Checked at the plan's base in a scratch crate depending on
`jig-domain-client` (whose contract jig 11 moves into
`jig-client-domain`), `skein-lib` and `skein-ui`: `heavy cargo clippy -p
w7-sketch --all-targets -- -D warnings` clean, the bodies
`unreachable!("sketch: ...")` under `#![expect(unused_variables, ..)]`.
`Ask` is jig 14's sketch's; `Me`, `Offered`, `Effect`, `Creating`,
`WaitsOn`, `WaitedFor`, `Whom` are jig 22's; `Key` is jig 12's, stubbed. Checked again at the ready
check against `main`.

```rust
// crates/jig-client/src/facts.rs
/// What a chat is doing, in pages.md, 4's table order; `words` says each.
pub enum ChatStatus {
    /// Due, preparing or claimed.
    Starting,
    /// Running, writing.
    Writing,
    /// Running a tool, named when its host names it, since then.
    Tool { tool: Option<Text>, since: Wall },
    /// Waiting on the reader, or a role they hold in the task's project: for what (the oldest entry's kind), and how many.
    WaitingForYou { kind: WaitedFor, entries: u32 },
    /// Waiting on another person, or a role the reader does not hold.
    WaitingFor(Whom),
    /// Idle, parked: words resume it.
    Parked,
    /// Backing off: when, which try of how many, and why.
    Retrying { at: Wall, try_number: u32, tries: u32, class: FailureClass },
    /// Held, and why.
    Held(HoldCategory),
    /// Closing.
    Closing,
    /// Ended, and when.
    Ended { at: Wall },
}
/// A list row's or a page's: a task line's `chip`, `waits_on` and `project`, or a task document's.
pub fn chat_status(chip: &Chip, waits_on: &WaitsOn, project: u32, me: &Me) -> ChatStatus;
pub fn effect(offers: &[Offered], act: ActKind) -> Option<&Effect>;
pub fn offer(offers: &[Offered], act: ActKind) -> Option<&Admission>;
pub fn dangerous(act: ActKind) -> bool;

// crates/jig-client/src/words.rs
/// The owner of `words`' phrases, beside the reference UI's own (65 536).
pub const OWNER: skein_ui::Owner = skein_ui::Owner { number: 65_537 };
/// One phrase of jig's enums.
pub enum Word { Phase(Phase), Hold(HoldWord), Refusal(RefusalWord), Waiting(WaitingWord), Status(StatusWord) }
impl Word {
    pub fn id(self) -> PhraseId;
    pub fn hold(value: &HoldCategory) -> Self;
    pub fn refusal(value: &Refusal) -> Self;
    pub fn waiting(value: &WaitsOn) -> Self;
    pub fn status(value: &ChatStatus) -> Self;
}
/// Each word's shape, its holes bounded by `text` bytes.
pub fn phrase_shapes(text: u32) -> Box<[PhraseShape]>;
/// jig's English for each word, checked by skein-ui when loaded.
pub fn english() -> Language;

// crates/jig-client/src/shapes.rs
/// jig's chat and goal templates' numbers (contract.md, 3.6).
pub const CHAT: u32 = 1;
pub const GOAL: u32 = 2;
/// Why a shape gives no request: the client sends nothing.
pub enum NotOffered { Project, Template, Executor, Ceiling, Creating }
/// What becomes of one live delegate of a closing chat.
pub enum Fate { Cancel, Move }
pub struct Delegate { pub task: u64, pub fate: Fate }
/// What the person wrote in a goal's form.
pub struct GoalDraft { pub words: Text, pub executor: Executor, pub budget: u64, pub deadline: Option<Wall>, pub priority: Option<u32> }
/// What a goal's draft will be, said before anything is sent.
pub enum Judged { Create(Ask), Propose { ask: Ask, to: Recipient }, Refused(Refusal), NotOffered(NotOffered) }
/// Where a draft falls against a project's creating bounds.
pub enum Fall { Within, Proposable(Recipient), Refused(Refusal), NotOffered(NotOffered) }
pub fn chat(me: &Me, project: u32, words: Text) -> Result<Ask, NotOffered>;
pub fn close_chat(me: &Me, project: u32, chat: u64, delegates: &[Delegate]) -> Result<Ask, NotOffered>;
pub fn set_goal(me: &Me, project: u32, draft: GoalDraft, reason: Text) -> Judged;
pub fn template(me: &Me, project: u32, number: u32) -> Option<&Template>;

/// contract.md, 5.3's comparison, easy to get wrong.
pub fn fall(creating: &Creating, draft: &GoalDraft) -> Fall {
    let within = &creating.within;
    if !listed(within, &draft.executor) {
        return Fall::NotOffered(NotOffered::Executor);
    }
    if draft.budget > within.maximum_budget || !no_later(draft.deadline, within.maximum_deadline) {
        return Fall::NotOffered(NotOffered::Ceiling);
    }
    if draft.budget <= within.budget && no_later(draft.deadline, within.latest) {
        return Fall::Within;
    }
    match &creating.beyond {
        Beyond::Proposed { to } => Fall::Proposable(to.clone()),
        Beyond::Refused(why) => Fall::Refused(why.clone()),
    }
}
/// An agent's charter among `charters`, or a procedure among `procedures`; a person or a role never.
fn listed(within: &CreateBounds, executor: &Executor) -> bool;
/// A task with no deadline is beyond any bound that has one.
const fn no_later(deadline: Option<Wall>, bound: Option<Wall>) -> bool {
    match (deadline, bound) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(at), Some(latest)) => at.as_nanos() <= latest.as_nanos(),
    }
}

// crates/jig-client/src/thread.rs
/// The thread's own bounds, given by the application.
pub struct Limits { pub lines: u32, pub bytes: u64, pub text: u32 }
/// A person's words, in their three states.
pub enum WordsState { BeingSent, Sent { message: u64 }, Read }
/// One line of the thread, oldest first.
pub enum Line<'a> {
    Words { by: u64, words: &'a [u8], state: WordsState },
    Turn { name: TurnName, turn: &'a Turn },
    Undecoded { name: TurnName, why: TurnProblem },
    Provisional { name: ProvisionalTurnName, text: &'a [u8], cut: bool },
    RunBegan { attempt: u64 },
    RunEnded { attempt: u64 },
    Left { lines: u32 },
}
/// A conversation's thread: committed turns, words and provisional text, merged in order across windows and gaps.
pub struct Thread { /* the moved model's fields */ }
impl Thread {
    pub fn new(task: u64, limits: &Limits) -> Self;
    pub fn document(&mut self, task: &Task) -> Option<Read>;
    pub fn page(&mut self, page: ActivityPage) -> Option<Read>;
    pub fn older(&mut self) -> Option<Read>;
    pub fn run(&mut self, run: &RunState, text: Option<&RunText>);
    pub fn sending(&mut self, key: Key, by: u64, words: Text) -> bool;
    pub fn answered(&mut self, key: Key, answer: &Answer);
    pub fn lines(&self) -> impl Iterator<Item = Line<'_>>;
}

// testing/jig-client-test-kit/src/check.rs: ordinary Rust; `Limits` the check's own (objects, revisions per object).
/// What a decision's revision is the revision of (contract.md, 5.2).
pub enum Object { Task(u64), Proposal(ProposalKey), Escalation(u64), PersonTask(u64), Note { project: u32, note: u64 } }
/// A document the client delivered to the application, as the referee saw it.
pub enum Delivered<'a> { Task(&'a Task), Proposal(&'a OwnerProposal), Escalation(&'a OwnerEscalation), PersonTask(&'a OwnerPersonTask), Note(&'a Note) }
/// A breach of jig's check.
pub enum Fault {
    /// A decision named a revision no delivered document of its object carried before it was sent.
    Undelivered { key: Key, object: Object, revision: Revision },
    /// A deciding act on a tree that did not show its decision's facts.
    Unseen { key: Key, act: ActKind },
}
/// jig's check (testing.md, 6.2): checked, never mandated.
pub struct Check { /* per object, the revisions delivered, bounded */ }
impl Check {
    pub fn new(limits: &Limits) -> Self;
    pub fn delivered(&mut self, document: Delivered<'_>);
    pub fn sent(&mut self, key: Key, ask: &Ask) -> Option<Fault>;
    pub fn acted(&mut self, key: Key, act: ActKind, tree: &Tree, node: NodeId, facts: &[&[u8]]) -> Option<Fault>;
}
/// The object and revision a decision names, if it names one.
pub fn named(ask: &Ask) -> Option<(Object, Revision)>;
```

Each helper is owned by this session; jig 16's reference UI and the
application downstream use them by name. None holds state but `Thread`,
which the application holds and feeds.

## Questions to raise when reached

- None the design leaves open that these increments decide.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep -c
  '^jig-15\.'` prints 6.
- `git -C ~/src/rust/jig ls-tree --name-only origin/main
  crates/jig-client/src/ | grep -c -E '^crates/jig-client/src/(facts|words|shapes|thread)\.rs$'`
  prints 4.
- `git -C ~/src/rust/jig grep -n -E 'jig_client_view' origin/main --
  crates/jig-client/src` prints nothing.
