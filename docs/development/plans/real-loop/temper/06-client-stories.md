# Session 06: the client stories over the tools

`tests/client`, the crate `temper-client-world`, gains a second client
beside the fake person: the tools' root (`temper-client-tools`'s
`root::Root`) as a scripted agent making calls against temper's engine
root on the `Joint` (tools.md, 11.2; testing.md, 6.3;
`client/README.md`, 10), with area 1's stories and 11.2's faults, and
7 and 8.2's unkeyed act cut before its answer. Their process tier is
temper 08, 09 and 10's.

Needs at start: nothing outside this session: the tools plan's sessions
(the table, the root, the waits, the forge's parts, help) are on
`origin/main`. 06.2 to 06.5 need 06.1.
Steps: `temper-06.1`, `temper-06.2`, `temper-06.3`, `temper-06.4`,
`temper-06.5`.

## Read first

- temper's `docs/design/client/tools.md`: 4, 5.2, 5.4, 5.6, 6.1, 6.3, 7, 8.2,
  11.2, 12; `testing.md`: 5.1, 6.1, 6.3. jig's
  `docs/design/client/domain/testing.md`: 6.1 (Made once), 6.2 (jig's
  check). The tools plan's `decisions.md`: question 29's correction, 33.
- The code: `tests/client/src/app.rs` (`Temper`, `Event`), `front.rs`
  (`Joint`: `new`, `with_questions`, `bind`, `fault`, `reload`,
  `request`, `incoming`, `packet`), `fixtures.rs` (`world`, `codec`,
  `chats`, `goal_with_held_delegate`, `QUESTION_WORKER`), `translate.rs`
  (`delivery_packet`), `tests/{stories,task,chat}.rs`;
  `crates/temper-client-tools/src/{root,limits,table,forge}.rs`,
  `jig/{runner,call}.rs` (`Printed`, `Call`, `Parsed`),
  `jig/render/{outcome,acts,turn}.rs`, the goldens `turn_part.txt` and
  `activity_turn.txt`; jig's `tests/client/src/harness/{world,mod}.rs`
  (`client_with`, `reload`, `kept`, `Until`, `Front`, `FrontFault`) and
  `tests/client-protocol/src/client/thread.rs` (`Story::turn_row_costs`)
  with `tests/turn_read.rs`; temper's `tests/engine/domain/src/
  {fixtures,store,protocol}.rs` (`forge_limits`, `Store::with_limits`,
  `turn`), `tests/worker/domain/src/protocol.rs` (`session_turn`,
  `endpoints`), `crates/temper-service/src/limits.rs` (`profile`,
  `Root`), `crates/temper-engine-domain/src/limits.rs` (`LOAD_BYTES`).

## What the design asks

- **11.2, the stories:** a chat started (06.1); its reply waited for, a
  message, the chat closed (06.2); a turn too large for a page read
  whole (06.3); a repository adopted, a held task released (06.4).
- **11.2, the faults:** "a call cut by a link drop or an engine restart,
  asked again under its named key, is made once" (06.5): the link drop
  is `FrontFault::NoAnswer`, the restart `FrontFault::Restart`; and per
  7, "a rerun from any directory signed in as the party is the same
  request".
- **7 and 8.2, the unkeyed act:** the table draws a key, the small store
  keeps it until answered, and after a cut "the next call that sends
  sends it again under its key", printing its `kept:` line (06.5).
- **6.1, a kept line:** "`kept: `, the request's operation, ..., then
  its outcome as an act's prints ... No name and no key" (the tools
  plan's question 29's correction and 33). A test asserts the whole line.
- **5.4:** "a wait begun after chat or message answered is not met by an
  earlier reply" (06.2). **testing.md 5.1:** the large turn's values are
  the profile's, by name, with the reason (06.3).
- **Not here:** `pending` is the host's (5.6; temper 05); `dropped:` is
  `temper-client-shell-cli`'s (ruling 130); the other party's parked line cannot be made, as
  `Joint::bind` binds one deployment owner: its lines are the tools
  plan's temper-03.4 units, and temper 09 leaves the story for later.

## Choices this plan makes

1. **One App, two arms.** `app::Root` is `Person(Domain)` or
   `Agent(Box<agent::Agent>)`; `app::View` is `Person(View)` or
   `Agent { render, limits }`; `app::Event` gains `Call { number, call,
   json }`. The tools' `Request::Client { request }` passes on as
   `temper_client_domain::Request::Client { request }`, `Fact` as `Fact`,
   so `duty`, `sent`, `unsent`, the `Joint` and `translate` serve the
   agent unchanged; `Ready`, `Kept` and `Printed` stay on `agent::Agent`.
   The agent's arm steps `root::step` under `root::Limits { domain:
   env.limits, tools: limits::tools() }`: `Event::Start` steps
   `StartUnlinked { kept }`, then `SignIn { with: With::Transport }`, as
   the person's arm does; `domain::Event::Client` goes to the root, `Went` is ignored,
   any other variant panics, named. `take` does nothing, `build` shows
   only the link (below), `act` returns `None`; `max_out` adds the runner's `2 × rows`; `worst_case`
   stays one function of the limits (Questions): `domain::worst_case`
   of the limits it takes, plus twice `view::worst_case` at
   `fixtures::domain_limits()`, the person's preset, whose view bound
   the tools' preset overflows (ruling 103). The agent's
   `View::Agent { render, limits }` is `skein_ui::View::new(&limits)` at
   that preset's tree bound, and its tree is the link alone, as jig's
   referee meets `Online` and `Offline` only from `Seen::Link` (ruling
   108): `Page` > `Banner` > `Region` at `view::frame::link_region()` >
   one `Status` named `Live` for `Link::Live`, `Offline` for
   `Link::Offline`, none otherwise, then a `Main` with one keyed `Item`
   per task whose `release` act a printed call offered (ruling 114),
   so that `skein_ui::outline::check` holds; `build` rebuilds it when that state
   changes (`render.stale`), from `root.client().link().state`, and the
   two names are consts that `Temper::seen` matches too. No bindings.
2. **The agent's limits are the tools' preset.** `fixtures::agent` gives
   `Joint::next_client_limits` `temper_client_tools::limits::client()`,
   then `client_with(PARTY, inbox(), that, agent::new)`. The `Joint`
   translates each client's traffic under its own limits and, from
   06.3, decodes its turns under `fixtures::codec_for` of them; the
   person's codec is unchanged.
3. **A process is a root.** `world.reload(agent)` is the next process
   that sends, started with DIRECTORY's kept records, `world.kept(agent)`
   (8.2, step 2); a second `fixtures::agent` is another DIRECTORY of the
   same party. The link's 30 s deadline is the host's (temper 05).
4. **8.2's step 4 is the fixture's:** `fixtures::give` gives a call once
   the root has pushed `Ready` and `Root::kept()` is 0.
5. **The large turn.** `Joint::with_large_turns` is `Joint::new` on
   `Store::with_limits`, with `journal.transcript_bytes` and
   `loads.{bytes, reply_bytes, transcript_bytes}` copied by name from
   `temper_service::limits::profile(Profile::Small).root` onto
   `forge_limits()`, 11.2 cited beside them. The turn is `session_turn`
   text of `LARGE_TURN_TEXT` plus one `llm::Block::Opaque` of
   `LARGE_TURN_OPAQUE`, encoded by `smith_protocol_channel::encode_turn`
   as `transcript_turn` does, committed by `protocol::turn(
   QUESTION_WORKER, ..)`. The opaque size is measured so that a lone
   read of the turn is at most `LOAD_BYTES` and its activity row more
   (jig 03's overlap).
6. **The agent copies printed acts:** a release from `task D`'s acts,
   `turn T A.N` from a size line; it builds nothing a person could not
   read.

## Rules that bind it

- **jig's check and Made once** hold through `Verdict::Passed`, which
  each fixture run asserts (jig's testing.md, 6.1, 6.2).
- **Limits from the builders:** the agent's from `temper_client_tools::
  limits::{client, tools}`, the person's as today, the large turn's from
  the profile by name (testing.md, 5.1). None typed.
- **No sleeps:** `Until::Steps` or `Until::Done`, each run within
  `fixtures::WITHIN` of world time.
- **Untouched:** `translate/forge.rs`; the person's tests but
  `tests/chat.rs`'s reach into its root; anything upstream; and
  `tests/system/src/lib.rs`, whose stale line 2 stays, as no step
  touches the file (report).
- **No STATUS edits.**

## Increments

### 06.1 The agent beside the person

Scope: the agent's arm, its fixture and the first story, a chat started
(tools.md, 11.2; Choices 1 to 4).
Needs: nothing outside this session.
Files: `tests/client/Cargo.toml` (`temper-client-tools.workspace =
true`; the root entry exists, so only `Cargo.lock` changes: hotspot,
forge kit); `tests/client/src/{app.rs, agent.rs (new), lib.rs, front.rs,
fixtures.rs}`; `crates/temper-client-tools/src/root.rs` (`Root::unlinked`,
ruling 107); `tests/client/tests/chat.rs` (`world.root(..).person()`,
`Probe` holds `app::Root`); `tests/client/tests/agent.rs` (new).
Shape:
- `agent::Agent` holds `root::Root`, `ready`, the kept lines, `Printed`
  by number and `calls`; `agent::new` builds it by `root::Root::unlinked(
  &agent::limits(), seed)`: `Root::new`'s `Client::new` holds a link,
  which `StartUnlinked` refuses. `Root::unlinked` is `Root::new` on
  `Client::unlinked`, its errors the same (ruling 107).
- `Joint` keeps a `BTreeMap<ClientId, client::Limits>`:
  `next_client_limits` sets what the next `bind` takes, other clients
  keep `self.limits`; `request` and `packet` pass that client's
  `client.domain` to `translate`.
- `fixtures::call(words)`: `table::parse`'s `Parsed::Call` as
  `app::Event::Call`; a usage error panics, named. `fixtures::give`
  numbers its call `Agent::calls() + 1`. `fixtures::answered` runs
  `Until::Steps(2)` until `Printed(number)`, bounded by `ANSWER_STEPS`,
  and asserts the verdict. `fixtures::done_task` reads `done: task N, `.
Test shape: `tests/agent.rs`, focused, under 0.5 s.
```rust
#[test]
fn a_chat_the_agent_starts_is_seen_by_the_person() {
    let mut world = fixtures::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let number = fixtures::give(&mut world, agent, &[b"chat", b"--project", b"1", b"--key", b"hello", b"Hello"]);
    let printed = fixtures::answered(&mut world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    let task = fixtures::done_task(&printed);
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(person, vec![Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}
```
Proof: joint (temper's engine root and the tools' root on jig's client
world), no higher than its test: the process tier belongs to temper 08,
09 and 10.
Don't: give the tools' root a `worst_case` (tools plan's crate); use the
person's limits for the agent's root or its `Joint` traffic (its
presentation allowance and link tree's bound are the person's, rulings
103 and 108); route by `Request` variants the person lacks.
Size: at most ~285 lines of `tests/client/src`, by the plan's command
over that directory alone (`git diff -M --numstat origin/main...HEAD --
tests/client/src`, summed by awk); 0 by part 1's count, which leaves out
`tests/`.
Removes: nothing.

### 06.2 A chat's life: its reply, a message, its close

Scope: 11.2's reply waited for, message and close, with 5.4's `replied`
and `ended`.
Needs: 06.1's agent.
Files: `tests/client/tests/agent.rs`; `tests/client/src/front.rs`
(`park_runs`, ruling 113); `crates/temper-client-tools/src/jig/wait.rs`
and `jig/render.rs` (an ended task's historical read, ruling 121).
Shape: as `stories.rs`'s answered chat, each test begins with
`world.front().engine.hello(Token::new(7))` and `park_runs = true`
(ruling 113): `Joint.park_runs`, beside `answer_runs`, answers each
assignment as `answer_runs` does but with `End::Parked`: a chat's run
done for now (tools.md, 5.4), where `Finished` ends the chat and drops
it from the reads; `answer_runs` and its stories are unchanged. An
assignment that delivers messages is first answered by one committed
turn whose `read` is its last message, through
`temper_engine_domain_world::protocol::turn` with an opaque body, as
`fixtures::replied_chat` commits, then parked: a park's read may not
pass the last committed turn's (jig's `routing/run_payloads.rs`;
ruling 115);
then `chat` and `wait task T --until replied`; after that reply,
`message T More --key more` and a new wait, met only by the run that
answers it; `close T --key bye` and `wait task T --until ended`.
Test shape: `tests/agent.rs`: `the_agent_waits_for_its_chats_reply`,
`a_message_is_followed_by_its_reply_waited_for`,
`a_chat_the_agent_closes_ends`; each focused, under 0.5 s, asserting
each call's status and `done: task T, <operation>`.
Proof: joint, no higher than its test (06.1).
Don't: commit a turn but the one that reads an assignment's messages,
or decode its body; the run's park is the reply (rulings 113, 115).
Size: at most ~20 lines of `tests/client/src`, by the same command;
~30 by part 1's count (ruling 121).
Removes: nothing.

### 06.3 A turn too large for a page, read whole

Scope: 11.2's large turn. The page bound is passed while a lone read of
the turn fits (tools.md, 5.2 and 12; Choice 5).
Needs: 06.1's agent.
Files: `tests/client/Cargo.toml` (`smith-domain-session`,
`smith-protocol-channel`, `smith-transcript`, each `.workspace = true`,
and `temper-worker-domain-world = { path = "../worker/domain" }`, as
`tests/system/Cargo.toml` has the first two and the path, and
`tests/worker/domain/Cargo.toml` `smith-transcript`) and `Cargo.lock`;
`tests/client/src/{front.rs, fixtures.rs, translate.rs}`;
`tests/client/tests/agent_turn.rs` (new); `crates/temper-engine-domain/src/lib.rs`
(`LOAD_BYTES` joins `pub use limits::{..}`, ruling 113).
Shape:
- `Joint::with_large_turns(seed, &client::Limits)` (Choice 5);
  `fixtures::large_turn(front, task)` commits the turn, returning
  `(attempt, number)`; `fixtures::turn_row_costs(front, task)` builds
  `(single, activity)` from the store's turn record and owner as
  `Story::turn_row_costs` does, by `jig_domain_core::reads::Row::
  owned_bytes`.
- `fixtures::codec_for(&jig_client::DomainLimits)` derives the codec as
  jig's link's `translate::turn::decode` does (crate-private there):
  `TurnLimits` from `rows` and `text`, and bytes the lesser of the wire's
  `turn_carried_bytes` and `document_bytes`. `delivery_packet` takes it
  per client.
- `fixtures::read_whole(world, agent, words)` gives the call, follows
  each `read with:` or `read on with:` call, found mid-line (a cut part
  ends `N bytes left; read on with: turn …`, the tools' `render::turn::words`;
  the size line says `read with: turn …`, `render::turn::sized`), and returns the joined text and the reads.
- `LARGE_TURN_TEXT` 24_576: past `limits::PART` (16 KiB), within
  `limits::TEXT` (32 KiB): two parts, decoded whole. `LARGE_TURN_OPAQUE`
  as measured; the commit body gives `(single, activity)` and
  `LOAD_BYTES`.
Test shape: `tests/agent_turn.rs`, each focused, under 0.5 s, from
`fixtures::world_with_large_turns(SEED)`:
`a_large_turns_rows_overlap_the_page_bound`;
`a_large_turn_in_a_chat_prints_as_its_size_line` (`chat T` holds `read
with: turn T A.N`); `a_large_turn_is_read_whole_by_its_parts` (two
reads; the joined text is the committed one).
```rust
#[test]
fn a_large_turns_rows_overlap_the_page_bound() {
    let mut world = fixtures::world_with_large_turns(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"Hello");
    fixtures::large_turn(world.front(), task);
    let (single, activity) = fixtures::turn_row_costs(world.front(), task);
    assert!(single <= temper_engine_domain::LOAD_BYTES, "{single}");
    assert!(temper_engine_domain::LOAD_BYTES < activity, "{activity}");
}
```
Proof: joint, no higher than its test (06.1). The bodies are about
64 KiB: measure the three tests and give their medians in the commit
body (Questions).
Don't: lower `LOAD_BYTES` (fixed in `core_limits`) or a page bound below
the profile's; type a size the profile names.
Size: at most ~150 lines of `tests/client/src`, by the same command; 0
by part 1's count.
Removes: nothing.

### 06.4 A repository adopted, a held task released

Scope: 11.2's adoption and release (Choice 6).
Needs: 06.1's agent.
Files: `tests/client/src/{fixtures.rs, agent.rs}`, `tests/client/tests/agent.rs`.
Shape: `fixtures::printed_act(stdout, act)`: the words of the first
`acts:` line naming `act`. The release test calls
`fixtures::goal_with_held_delegate` before `fixtures::agent`, as it
replaces the front; the agent gives `task D`'s `release D --hold R --key NAME`, as printed,
its own key for NAME. `Agent::build`'s `Main` shows, for each task D a
printed call's `acts:` line offers `release D` for, an `Item` keyed
`Key { owner: frame::owner(), kind: 2, a: D, b: 0 }` (cards.rs's
escalation), nested as the inbox nests its entries, with no `Hold
revision` name: the referee then compares the hold sent with the
revision the agent's read delivered (ruling 114).
Test shape: `tests/agent.rs`, each focused, under 0.5 s.
- `a_repository_the_agent_adopts_is_held_by_the_engine`: `adopt
  --project 1 --owner acme --name shop --landing main --key shop` (as
  `adopt::ROW` declares it) exits 0; `engine_adopted(1, 7)`;
  `applications_admitted() == 1`.
- `a_held_task_the_agent_releases_from_the_printed_act_runs_again`:
  `done: task D, release`; `engine.task(D).tries == Tries::NONE`, as
  `task.rs`'s person story asserts.
Proof: joint, no higher than its test (06.1).
Don't: touch `translate/forge.rs`; print a revision in the agent's
tree, or set `Sent`'s card or revision for the agent apart (ruling 114).
Size: at most ~45 lines of `tests/client/src`, by the same command; 0
by part 1's count.
Removes: nothing.

### 06.5 Cut calls, made once

Scope: 11.2's faults over a chat and over the forge's owner, and 7 and
8.2's unkeyed act (Choice 3).
Needs: 06.1's agent.
Files: `tests/client/src/{front.rs, fixtures.rs}`,
`tests/client/tests/agent_recovery.rs` (new); `crates/temper-client-tools`'
runner, its `Me` read opened again after a `SignedOut` end (ruling 122).
Shape: `Joint::answers_to_lose()`: the count `FrontFault::NoAnswer`
arms. `fixtures::cut(world, agent, words)` arms `NoAnswer`, gives the
call, runs `Until::Steps(2)` until `answers_to_lose()` is 0 (bounded by
`ANSWER_STEPS`), and asserts world time is before the client's first
retry (`limits::client()`'s `first`), so the process ends unresent.
Test shape: `tests/agent_recovery.rs`, each focused, under 0.5 s, each
cutting the call it names.
- `a_chat_cut_by_a_lost_answer_is_resent_with_its_kept_line`: `cut`, then
  `reload`, then `chat --key second Again`. The kept lines are exactly
  `kept: chat, done: task T, chat\n`, and `chats == 2`.
- `a_chat_cut_by_a_lost_answer_asked_again_under_its_name_is_made_once`:
  the same call after `reload` prints `done: task T, chat`, with the same
  T, and `chats == 1`.
- `a_chat_cut_by_an_engine_restart_asked_again_from_another_directory_is_made_once`:
  `cut`, then `FrontFault::Restart { offline }` before any rerun, then
  `world.reload(agent)`, as jig's world ends no client (ruling 122), then
  a second `fixtures::agent` gives the same call: same T, `chats == 1`.
- `an_adoption_cut_by_a_lost_answer_asked_again_under_its_name_is_made_once`:
  the same `adopt` cut, then `reload`, then the same call:
  exit 0, and `applications_admitted() == 1`.
- `an_unkeyed_stop_cut_before_its_answer_is_resent_with_its_kept_line`:
```rust
#[test]
fn an_unkeyed_stop_cut_before_its_answer_is_resent_with_its_kept_line() {
    let mut world = fixtures::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let number = fixtures::give(&mut world, agent, &[b"chat", b"--project", b"1", b"--key", b"hello", b"Hello"]);
    let task = fixtures::done_task(&fixtures::answered(&mut world, agent, number));
    let stop = format!("{task}");
    fixtures::cut(&mut world, agent, &[b"stop", stop.as_bytes()]);
    assert_eq!(world.kept(agent).len(), 1, "the drawn key's record is kept");
    world.reload(agent);
    let number = fixtures::give(&mut world, agent, &[b"chat", b"--project", b"1", b"--key", b"again", b"Again"]);
    assert_eq!(fixtures::answered(&mut world, agent, number).status, 0);
    let expected = format!("kept: stop, done: task {task}, stop\n");
    assert_eq!(world.root(agent).agent().kept_lines(), [expected.into_bytes().into_boxed_slice()]);
}
```
Proof: joint, no higher than its test: a signal's cut and DIRECTORY on
disk belong to temper 09 and 10.
Don't: stop a run at a wall time; let a retry land before `reload`.
Size: at most ~40 lines of `tests/client/src`, by the same command; ~5
by part 1's count (ruling 122).
Removes: nothing.

Each test's cost: the crate's focused tests take 0.79 s together at
the base (median), about 20 ms each (this plan's `findings/base-times.md`); the
14 new ones should add 0.3 to 0.6 s to the worlds' sum below the
domain, 3.91 s at the base against testing.md 6.1's 4 s, which holds
no test within its guide (README, Rules, "The worlds' sum").

## Interface sketches

```rust
// tests/client/src/app.rs
pub enum Root { Person(temper_client_domain::Domain), Agent(Box<crate::agent::Agent>) }
pub enum View { Person(temper_client_view::View), Agent { render: skein_ui::View, limits: skein_ui::Limits } }
impl Root {
    pub fn person(&self) -> &temper_client_domain::Domain;
    pub fn agent(&self) -> &crate::agent::Agent;
}
// `Event` gains: Call { number: u32, call: temper_client_tools::jig::call::Call, json: bool }

// tests/client/src/agent.rs
pub struct Agent { /* root::Root, ready, kept lines, Printed by number, calls */ }
impl Agent {
    pub fn ready(&self) -> bool;
    pub fn kept(&self) -> u32;
    pub fn calls(&self) -> u32;
    pub fn kept_lines(&self) -> &[Box<[u8]>];
    pub fn printed(&self, number: u32) -> Option<&temper_client_tools::jig::Printed>;
}
pub fn limits() -> temper_client_tools::root::Limits;
pub fn new(seed: u64) -> (crate::app::Root, crate::app::View);
pub fn step(agent: &mut Agent, env: &skein_lib::Env<temper_client_domain::Limits>, event: crate::app::Event, out: &mut skein_lib::Queue<temper_client_domain::Request>);

// tests/client/src/front.rs
impl Joint {
    pub fn next_client_limits(&mut self, limits: temper_client_domain::Limits);
    pub fn with_large_turns(seed: u64, limits: &temper_client_domain::Limits) -> Self;
    pub fn answers_to_lose(&self) -> u32;
}

// tests/client/src/fixtures.rs
/// Two 16 KiB parts, within the client's 32 KiB text per turn (tools.md, 12).
pub const LARGE_TURN_TEXT: usize = 24_576;
/// An estimate 06.3 replaces by its measure: a lone read of the turn fits `LOAD_BYTES`; its activity row does not.
pub const LARGE_TURN_OPAQUE: usize = 38_912;
/// The `Until::Steps(2)` runs `give` waits for a call's answer before it fails: a loop's guard, not a limit.
/// One `Steps(1)` run stops before world time moves (jig's `World::run`), so it never reaches a callback (ruling 108).
const ANSWER_STEPS: u32 = 4096;
pub fn agent(world: &mut World<Temper, Joint>) -> ClientId;
pub fn call(words: &[&[u8]]) -> crate::app::Event;
pub fn give(world: &mut World<Temper, Joint>, agent: ClientId, words: &[&[u8]]) -> u32;
pub fn answered(world: &mut World<Temper, Joint>, agent: ClientId, number: u32) -> temper_client_tools::jig::Printed;
pub fn done_task(printed: &temper_client_tools::jig::Printed) -> u64;
pub fn printed_act(stdout: &[u8], act: &[u8]) -> Vec<Box<[u8]>>;
pub fn cut(world: &mut World<Temper, Joint>, agent: ClientId, words: &[&[u8]]);
pub fn read_whole(world: &mut World<Temper, Joint>, agent: ClientId, words: &[&[u8]]) -> (Vec<u8>, u32);
pub fn codec_for(limits: &jig_client::DomainLimits) -> jig_protocol_transcript::Limits;
pub fn world_with_large_turns(seed: u64) -> World<Temper, Joint>;
pub fn large_turn(front: &mut Joint, task: u64) -> (u32, u32);
pub fn turn_row_costs(front: &Joint, task: u64) -> (u64, u64);
```

## Questions to raise when reached

- **The agent's heap bound:** `client_with` bounds it by
  `Temper::worst_case(&limits::client())`, the person's function; the
  tools' root has none. A failed heap check: raise it with the peak.
- **The raised limits** refused by `Driver::with_repositories` or the
  engine's validation on `forge_limits()`: raise it with the error.
- **No overlap:** no `LARGE_TURN_OPAQUE` gives single ≤ `LOAD_BYTES` <
  activity, or the client decodes the turn cut: raise it with the pair.
- **`release` not in `task D`'s acts:** follow the printed act where it
  is printed (`task G --tree`); raise it if nowhere.
- **A test over 0.5 s:** profiled, then made cheaper or moved to
  fuzzy, with the times; never merged. The worlds below the domain past
  4 s together hold no test within its guide (README, Rules, "The
  worlds' sum").

## Done when

Each prints what it says, on `origin/main`, after `git fetch`:

- `git grep -n -F 'temper-client-tools' origin/main -- tests/client/Cargo.toml`
  prints a line.
- `git grep -c -E 'fn (a_chat_the_agent_starts_is_seen_by_the_person|the_agent_waits_for_its_chats_reply|a_message_is_followed_by_its_reply_waited_for|a_chat_the_agent_closes_ends|a_large_turns_rows_overlap_the_page_bound|a_large_turn_in_a_chat_prints_as_its_size_line|a_large_turn_is_read_whole_by_its_parts|a_repository_the_agent_adopts_is_held_by_the_engine|an_adoption_cut_by_a_lost_answer_asked_again_under_its_name_is_made_once|a_held_task_the_agent_releases_from_the_printed_act_runs_again|a_chat_cut_by_a_lost_answer_is_resent_with_its_kept_line|a_chat_cut_by_a_lost_answer_asked_again_under_its_name_is_made_once|a_chat_cut_by_an_engine_restart_asked_again_from_another_directory_is_made_once|an_unkeyed_stop_cut_before_its_answer_is_resent_with_its_kept_line)\(' origin/main -- tests/client/tests`
  prints counts that sum to 14.
- `git grep -n -F 'Profile::Small' origin/main -- tests/client/src`
  prints a line.
- `git grep -n -F 'kept: stop, done: task' origin/main -- tests/client/tests`
  prints a line.
