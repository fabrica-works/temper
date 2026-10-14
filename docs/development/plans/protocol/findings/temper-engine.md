# Audit: temper-engine

2026-10-14, read only, by a sub-agent of the planner, at temper `cc2697cf` with jig `d91a89c4`, smith `dfc0fe57` and skein `b5d3a2c3` (temper's pins; jig's, smith's and skein's paths are relative to their pinned checkouts). Evidence the plan cites (development.md, 4.1 and 5.4): where it and landed code differ, the landed code wins. The design gaps it found are closed by the Markdown landing "Design: close the gaps the protocol plan's code audit found"; `README.md` here maps each finding to its increment.

I couldn't confirm three of the design's pieces against the code: the keyed adoption, the preparation's failure detail, and the saved-work branch. jig at the pin and temper's store design don't yet provide all of them, so those plan steps need a decision first. Everything below comes from the source at temper main cc2697cf and jig d91a89c. Graph coverage was clean for all 16 files I relied on, and I checked each against the source directly. Paths are relative to /home/free/src/rust/temper unless they say jig@pin. `ER` = crates/temper-engine-domain/src, `FT` = crates/temper-engine-domain-forge/src, `FC` = crates/temper-engine-domain-forge-client/src.

## 1. Engine root: what exists, and its shape
- **`Request::Forge { call: Token, repository: forge_client::api::Repository, op: forge_client::api::Op }`** — ER/boundary.rs:393. It is re-exported as `Output` (ER/lib.rs:34). Reads go out at once through `now` and writes after the commit as `Delivery::ForgeCall` (forge_route.rs:2026-2033).
- **`Event::Forge(Box<forge::Event>)`** — boundary.rs:229. It takes any forge event, not just the client's `Answered` and the forge's `Hint` (route.rs:75-82). A hint arriving before the root is ready is dropped.
- **`Event::Credential { number: u32, generation: u64, valid: skein_lib::Duration }`** — boundary.rs:224, mapped to the accounts event at route.rs:84.
- **`Parts { saved: Option<Box<[u32]>>, pushed: Box<[forge::Pushed]> }`** — boundary.rs:284. `Parts::none()` is at :295. **There is no `preparation` field.**
- **`forge::Pushed { tag: u32, commit: client::api::Commit }`** — FT/boundary.rs:196.
- **Store:** `Key { Core(..), Forge(temper_engine_domain_forge::Key) }` store.rs:215; `Range { Core(CoreRange), Census(..), Forge }` :270; `pub fn contains(&self, key: &Key) -> bool` :284 (a forge key only under `Range::Forge`, and `Forge` admits no core key); `Record::Forge { row: Box<Stored> }` :383; `Write::{Save(Record), Erase(Key)}` :402.
  - The forge's `Save` and `Erase` are routed at forge_route.rs:2009-2025.
- **Host calls:**
  - `HostRequest::{Decode { to: ReplyTo, task: u64, attempt: u64, call: fleet::Call }, Dropped { call }}` — host_route.rs:14.
  - `HostDelivery::{Render { to, key: CallKey, name, tool, answer: CallAnswer }, Answer { channel, task, attempt, name, call: SettledCall }}` — host_route.rs:23.
  - `WorkerInput::{DecodedHostCall { to, body: Call }, RenderedHostCall { to, answer: jig_domain_core::SettledAnswer }, ResultRead { payload: Token, end: tasks::End }}` — boundary.rs:301-310.
  - `WorkerRequest::ReadResult { payload: Token, outcome: Box<[u8]> }` — boundary.rs:441.
- **`Assignment`** — boundary.rs:24. Its fields are task, attempt, conversation, charter, run, sections, messages, `saved: Box<[u32]>` (tags that *start* from saved work), workstream, `workspace: ForgeWorkspace`, transcript, answered, settled and grants. **There is no saved-work branch to save to.** tests/system/src/composed/link.rs:32 says so and hard-codes `save: false`.
- **`ForgeRepository { tag: u32, provider: api::Repository, name, host, owner, start: ForgeStart, push: Option<Box<[u8]>>, identity: u32 }`** — :95. This matches the design, plus a `host` field.
- **`ForgeStart::{Base(b), Branch(b), Saved(b), Merge { branch, base: Commit }}`** — :86. **`ForgeWorkspace { repositories }`** — :109.
- **`Config { core, landing: LandingPolicy, forge: forge_client::Config, identities: Box<[forge::GitIdentity]>, forge_charter: u32 }`** — translate.rs:17.
  - `forge_client::Config { namespace, writers: Box<[Writer]> }` is at FC/identity.rs:112, and `Writer { forge: u16, author: u64 }` at :101.
  - The namespace from configuration is overwritten with `core.deployment` (ER/domain.rs:149-151). That matches "derived from the deployment's id".
- **Landing types:** `GitIdentity { forge: u16, credential: u32 }` lives in the forge crate (FT/boundary.rs:112), not the root. `LandingPolicy { deployment, projects: Map<u32, Box<[LandingRule]>> }` is at translate.rs:43. `LandingRule { connector, kind, pattern, enforce, ci, up_to_date, gates, approvals }` is at route/landing.rs:34.
- **Restart:** jig's `RestartStep::RestoreConnector { connector }` is handled at restart.rs:308-314 (it loads `Range::Forge`). The forge's `Event::Restored { clock }` (FT/boundary.rs:535) is pushed at restart.rs:262-264.
- **`Request::Busy(Box<forge::Event>)`** — boundary.rs:406, emitted while the root drains (stopping.rs:16).

## 2. The design's "Kept, with changes" items
**(a) Locate: not built.** `api::Read` has no `Locate` and `api::Answer` has no `Located`. The client crate's `Read` handling never checks the repository (FC/domain.rs:165-180), so the design's "repository 0" addressing would pass.
- Files that match on `Read` without a wildcard and would need an arm:
  - FC/api.rs, two shape matches near :479 and :550
  - FC/bounds.rs:73 (owner and name bytes)
  - ER/route/forge_route.rs:280-298 (`read_name`)
  - tests/engine/forge/client/src/translate.rs
- Files that match on `Answer` and would need an arm: FC api.rs, keep.rs, outbox.rs and bounds.rs; FT/capabilities.rs; FT/domain.rs:3259-3262; crates/temper-engine-smith/src/answers.rs; the forge client world's translate.rs, world.rs and referee.rs.
- The forge's adoption would gain a `Locate` stage before `Permission`, in `adopt` (FT/domain.rs:1058-1137) and `adoption_read` (:1152).
  - Every check that `adopt` makes today against the repository id has to move after the locate answer: existing repository, other project, a pending adoption of the same repository, capacity.
  - The root's `adoption_restore`, keyed by `adoption.provider`, also assumes the id up front (route/asks.rs:76-78).
- The fake forge domain names repositories by full name and has no API read that gives back an id (testing/temper-fake-forge-domain). The worlds map ids to names by hand: tests/engine/domain/tests/forge_routes.rs:425-431 maps `{1,2}` to "org/repo" and `{1,3}` to "org/extra".

**(b) Keyed adoption: not built.** Today it works like this:
1. The world calls `adopt_repository_ask(forge::Adoption, connector) -> Option<people::Ask>` (forge_route.rs:39). It packs a known repository id into the path `[forge u16 BE, repository u32 BE]` and puts home, host, owner, name, prefix, landing, ci and checks in `options`.
2. The world sends `PartyInput::Ask { reply_to, sign_in, key, ask: people::Ask::Adopt }` (boundary.rs:340).
3. The core's connector `Ask::Adopt` reaches `parse_adoption` (forge_route.rs:104), then `forge::Event::Adopt { reply_to, adoption }` (asks.rs:69-90).
4. The forge reads Permission → Branches → Settings → Protection → Collaborators (FT/domain.rs:17-23).
5. `Request::Adopted` is mapped to `people::Outcome` (forge_route.rs:2039-2120).

What jig at the pin provides for the design's flow:
- `core::Event::Application { reply_to, sign_in, key, project, connector, digest, owner }`, `ApplicationPart { request, ask: Box<people::Ask> }` and `ApplicationFinished { request, outcome: Result<Token, people::Again> }` — jig@pin crates/jig-domain-core/src/routing/vocabulary.rs:354-366.
- `core::Ask::{Application { request, token, owner, person, project, role, retention }, DropApplication { owner }, ApplicationPartAnswered { request, outcome }}` — vocabulary.rs:41-53.
- The people child accepts `Ask::Adopt` as an application part (jig@pin crates/jig-domain-people/src/domain.rs:1766-1835).

What temper lacks:
- The root marks these unreachable: `"temper's root admits no application request"` (asks.rs:38-40). held.rs:125 does the same for `Applied` and `ApplicationDelivery`.
- `PartyInput` has no application variant.
- Nowhere keeps the outcome under the token for the retention period (see Findings 3 and 4).

Where the work would go: boundary.rs (`PartyInput`), route.rs dispatch, asks.rs (both arms), and forge_route.rs (`adopt_repository_ask` would take the located id; the `Adopted` output would go to `ApplicationPartAnswered`).

**`forge::Adoption`** is at FT/boundary.rs:128: `{ project, home, provider: api::Repository, host, owner, name, prefix, role: Role, landing, ci, checks: Box<[u32]> }`.
- client.md 4.4's form leaves out the id and the host. The code needs both up front: `provider.repository`, and `host`, where empty is refused as `TooLarge` at FT/domain.rs:1059.
- `Role` has four variants: Owned, Adopted, Fork, Context. The form offers three, and `Adopted` makes `adopt_repository_ask` return None.
- An unconfigured forge is refused as `Forbidden` (domain.rs:1102). The design says "no such forge", but `api::Error` has no such variant and client.md 8's error-to-reason map has no such reason.

**(c) `Parts` carrying a failed preparation: not built.**
- It would go in boundary.rs:284/295 (a new field), ER/link.rs:32 (`parts_within`), the answer handling at link.rs:166-198, `result_event` at link.rs:203-245 (`let Parts { saved, pushed }` at :222), and `end()` (around :247, which drops jig's `Failed { detail }`).
- **jig at the pin has nowhere to put the words.** The core's `Event::AnswerPayload { run, attempt, payload, cumulative, end, saved, invalid_saved, read, refused }` (vocabulary.rs:452) has no detail, and `tasks::End::Failed(Class)` carries only a class.
- Tests that build `Parts` field by field would all break: tests/engine/domain/tests/forge_routes.rs (22 sites, lines 873-2973), root_routes.rs:3937, preparation.rs:56, tests/system/src/composed/link.rs:91 plus its tests at :263 and :272, and tests/conformance/src/link.rs:94. The 40 `Parts::none()` callers are unaffected.

**(d) Relaying the error answer for a call that didn't decode: not built, and silently dropped today.**
- `rendered()` (host_route.rs:111) has `let Some(key) = flight.key else { return };` at :115. A `RenderedHostCall` that arrives before `DecodedHostCall` is dropped and the run never gets an answer.
- The fix is to relay it as `Payload::SettledCall` plus `fleet::Event::Relayed`, the way the invalid branch of `decoded` already does (host_route.rs:66-82). That's roughly 8 lines.
- No existing test changes. The new test would go beside `typed_call` (root_routes.rs:4013).
- The engine world can't produce an undecodable call yet: an unknown name returns `Err` (tests/engine/domain/src/protocol.rs:176). The system world's `link::decode` calls `.expect` on decoding (composed/link.rs:116).

## 3. `crates/temper-engine-domain-forge`
- **`Stored`** (FT/boundary.rs:359) has 15 variants in this order: Call{key,answer}, ProposedEffect, Repository, Hold, Names{task,resources}, Subscription, BranchHead, PullState, Ci, Landed{commit,task}, Entry{entry,held}, Client(client::Stored), Change, Issue, Release. This **matches** store/README.md.
- **`Key`** (:413) has the same order and derives `PartialOrd, Ord`: Call(NamedCall{task,attempt,completion,position}), ProposedEffect(u64), Repository(api::Repository), Hold(Name), Names(u64), Subscription{task,topic}, BranchHead(Name), PullState(Name), Ci{repository,head}, Landed(Commit), Entry(u64), Client(client::Key), Change(u64), Issue(u64), Release(u64).
- **Names and topics:**
  - `Name { forge: u16, repository: u32, what: What }` :45.
  - `What::{Repository, Branch(Box<[Box<[u8]>]>), Pull(u64), Issue(u64)}` :52.
  - `Topic::{Landings{repository,branch}, Ci{repository,head}, Pull{repository,number}, Participation{repository,number}}` :212.
  - `client::Key::{Live(Resource), Repository(Repository)}` at FC/boundary.rs:196, where `client::What::Branch(Box<[u8]>)` is one byte string.
  - All of these match store/README.md section 3.
- **`Limits`** (FT/limits.rs:9): repositories, tasks, holds, subscriptions, entries, named_calls, changes, issues, resources_per_task, paths_per_subscription, name_bytes, output, facts, adoptions, collaborators, landings, judge_projects, judge_criteria, brief_sections, brief_bytes, issue_policy, change_policy, queue_window: Duration, client: client::Limits.
- **`effect_shape(kind: u16) -> Option<(EffectForm, Recovery)>`** (FT/lib.rs:70):
  - 2 and 11 are Transition/Unrecoverable.
  - 4 is Transition/Conditional; 3 is Creation/Conditional.
  - 5, 7, 8 and 9 are Creation/Unrecoverable.
  - 6, 10 and 12-15 are Set/Idempotent.
  - 0, 1 and 16+ are None, so kind 1 ("read") has no shape.
  - The numbering in the doc comment (:62-67) matches client.md 4.1.
- **`effect_key(deployment: &[u8], purpose: &EffectPurpose, max_bytes: u32) -> Option<Box<[u8]>>`** is in the forge *client* crate, FC/identity.rs:29.

## 4. Forge client API vs forge.md
- **`Read`** (FC/api.rs:49): the 18 variants and their fields match the 3.2 table. **`Locate { owner, name }` is missing.**
- **`Write`** (:97): all 14 match.
- **`Answer`** (:128): Items, Item, Pull, Reviews, Statuses, Remarks, Commit, Branches, PullFiles, Compare, Checks, Job, File, Protection(Option), Settings, Collaborators, Permission, Created, Commented, Reviewed, Merged, Branch(BranchCreation::{Created, Exists}), Done. **`Located { repository }` is missing.**
- **`Error`** (:178): all 18 of 3.4 are present (Unavailable, Timeout, RateLimited{after: Duration}, Forbidden, Missing, MissingJob, TooLarge, Empty, Full, Exists, NothingToMerge, Closed, Stale, Conflict, Protected, Refused, InvalidAnswer, Busy). They are in a different order from the design's table, which matters only if a codec numbers them by order.
- **`Hint { repository, change: Change, key: Option<Box<[u8]>> }`** :21. `Change::{Item(u64), Branch(b), BranchMoved{branch, head, actor}, Commit}` :28.
- **`File { path, before: Option<b>, after: Option<b> }`** :380.
- **`Event::Answered { call: Token, cost: u32, result: Result<Answer, Error> }`** — FC/boundary.rs:268.
- **The client's `Limits`** (FC/limits.rs:12) has calls, rows, answer_bytes, op_bytes, lifetime and repositories, plus pending, rate, reserve, window, inbox, read_attempts, backoff(_max), entries, write_attempts, clock_margin, resources, poll(_max), hinted, slow and facts.
- No other mismatches.

## 5. `crates/temper-engine-smith`
Public API:
- `start(assignment: &engine::Assignment, workspace: Option<run::Workspace>, transcript: Option<smith::Transcript>, reply_to: ReplyTo, host_run: Token, window: smith::Window) -> smith::Event` — start.rs:66
- `call(name: run::CallName, tool: &[u8], input: &run::HostInput) -> Result<engine::Call, Problem>` — calls.rs:80
- `tools(timeout: Duration) -> Box<[run::HostTool]>` — calls.rs:61
- `answer(answer: &CallAnswer) -> HostAnswer` — answers.rs:96. It is total: a render that fails comes back as a bounded error.
- `result(answer: run::Answer, change: Option<ChangeResource>) -> Result<Terminal{cumulative, turns, end}, Problem>` — finish.rs:69
- Also `turn`, `model` and `section`.
- `Problem::{Malformed, TooLarge, Missing, Type, Range, UnknownTool}` — lib.rs:46.

Callers: **only the system world** — tests/system/src/world.rs, composed/link.rs, goals.rs and forge/stories.rs. Neither the engine world nor the conformance world calls it. `start` returns a domain `Event::Start`. The world pulls the charter out of it, sets `charter.resume` itself, and calls `smith_protocol_channel::encode_charter` (composed/link.rs:154-160). So the resume rule lives in the world, not in this crate.

## 6. The worlds
**`jig-fake-store`** is used by:
- tests/engine/domain: src/store.rs:34-52 and :144
- tests/system: src/composed.rs:386 and :487-488, and tests/composed.rs:129 (`Fault::Hold`)
- tests/conformance: src/lib.rs:155 and :344-366, src/observations.rs:149

**The system world's own translation** (composed/link.rs):
- `assignment(&Assignment) -> (host::Assignment, wire::Workspace, Items)` :34. It hard-codes `save: false` and makes the remote as `owner/name` with no base.
- `parts(wire::Work, &[(u32, bool)]) -> engine::Parts` :72. It keeps tags and from-saved flags *in the world*, by position, and has no preparation.
- `decode` :109, `render` :121, `end` :129, the private `repository()` :229 and `charter()` :154.

channel.md 7 wants the service's functions to replace these. They don't exist yet, because `temper-service` isn't built.

**tests/conformance** plays the engine world's protocol stand and has its own `link.rs`, with a `Parts` literal at :94.

Which worlds change for each item:
- **`preparation` in `Parts`:** the engine world (the 24 literals above), the system world's `link::parts` and its tests, and conformance link.rs:94.
- **Locate:** every world that answers `Request::Forge`:
  - forge_routes.rs:411-445
  - tests/system/src/composed.rs `forge()` around :548 and tests/system/src/forge.rs
  - the forge client world's translate.rs `op` (:71) and `answer` (:186)
  - the fake forge domain, if it is to answer locates by name
- **Keyed adoption:** every caller of `adopt_repository_ask` or builder of `forge::Adoption`:
  - forge_routes.rs:611, :2109, :2165
  - tests/system/src/composed.rs:729 and forge.rs:480
  - tests/conformance/src/lib.rs:542
  - tests/engine/forge/src/world.rs, plus FT/tests.rs (2)
  - Each also checks `people::Outcome::Adopted`, which becomes `Applied { connector, token }` plus temper's outcome.

## 7. The large files
- **ER/route/forge_route.rs (2677 lines):**
  - Touched by (a): one arm in `read_name`.
  - Touched by (b): about 150-250 lines — `adopt_repository_ask` and `parse_adoption` (39-145, about 105 lines), the `Adopted` output (2039-2120, about 80), plus new application handling. (c) and (d) don't touch it.
  - Natural split: an `adoption` module (14-145 plus 2039-2120). Other seams are the workspace (1122-1395), the change procedure (1398-1960), `outputs` (1961-2358) and effects (2359-2642).
- **FT/domain.rs (3727 lines):**
  - Touched by (a)/(b): about 60-120 lines in the adoption section (1058-1322, 265 lines). It also gets one `Answer` arm at :3262, and a new arm in `restore` (:3580) if an outcome family is added.
  - Natural split: move the adoption section into `adoption.rs`, beside the existing capabilities.rs. Other seams are change (1792-2477), release (2477-2796), projection (2796-3002), topics and news (3090-3580) and restart (3580-3727).
- **tests/engine/domain/tests/root_routes.rs (4165 lines, 85 tests):**
  - Touched by (d): one new test of about 30 lines after `typed_call` (4013-4110).
  - Touched by (c): one literal at :3937. Not touched by (b).
  - Natural splits: the typed-call tests (4013-4110), people and roles (1-805), notes (1250-1498), delegation and proposals (1499-2365), restart and escalation (2365-3003), administration and policy (3004-3440).
- **tests/engine/domain/tests/forge_routes.rs (3021 lines, 31 tests):**
  - Touched by (a)/(b): about 150-200 lines — `adopt_with_checks` (596-635), the `Forge` handler (411-445), the collect for `Outcome::Adopted` (around 476-480), and the adoption tests (2003-2134, 2165-2192).
  - Touched by (c): 22 one-line literals, or none if a constructor is added.
  - Natural split: move the `World` harness (18-636) into the world crate's `src/`, and move the adoption tests (2003-2234) into their own file.

## Findings
1. **The root can't relay the error answer to a call that didn't decode** (host_route.rs:115 drops it, so the run waits forever). Blocks a step (design's 5(d)); the fix is about 8 lines.
2. **`Read::Locate` and `Answer::Located` don't exist; adoption needs the repository id up front.** Every check in `adopt` and the root's `adoption_restore` are keyed by the id. Changes a Shape: `forge::Adoption` loses the id; the client API, the forge's adoption stages and the root all change.
3. **The keyed adoption isn't built.** jig at the pin has the application request flow, but temper's root marks it unreachable (asks.rs:38-40, held.rs:125) and `PartyInput` has no variant for it. Blocks a step.
4. **store/README.md lists 15 families but none keeps a keyed adoption's outcome for the retention period.** jig requires the connector to keep it durably. Changes a Shape: a 16th `Stored`/`Key` variant at byte 15, its restore arm, and the store version.
5. **jig at the pin has nowhere to carry a failure detail** (`AnswerPayload` and `tasks::End::Failed(Class)` have none). So the root can't "put the preparation in words" in the attempt's failure detail. Blocks a step: it needs a jig change or a temper-owned record. `Parts.preparation` itself is also missing (changes a Shape).
6. **`Assignment` has no saved-work branch to save to**; the worlds hard-code `save: false`. So channel.md 4's `save` and its "both or neither" rule have no source. Changes a Shape.
7. **An unconfigured forge is refused as `Forbidden`, and an empty host as `TooLarge`.** The design wants "no such forge", which neither `api::Error` nor client.md 8's reason map has. Changes a Shape.
8. **`forge::Role` has a fourth variant, `Adopted`**, that the form doesn't offer and `adopt_repository_ask` refuses. Cosmetic: drop it or keep it.
9. **`Event::Forge(Box<forge::Event>)` accepts any forge event**, not only `Answered` and `Hint`. The protocol layer could inject `Adopt` or `Restore`. Changes a Shape if narrowed (low priority).
10. **The worker's tags are kept by the world, in order** (`link::Items`, composed/link.rs:28). The design has the worker name each repository by tag itself. That is worker-side, and it blocks reusing the service's `parts` function in the system world as channel.md 7 describes. Changes a Shape.
11. **`temper-engine-smith::start` returns a domain `Event::Start`**, and the charter's resume rule lives in the system world (link.rs:157). The service would have to repeat it. Changes a Shape: add a `charter(&Assignment)` function.
12. **The fake forge domain exposes no repository id or locate by name**, and the worlds map ids to names by hand. Blocks the step that adds locate in the worlds unless they answer locates from their own table.
13. **`GitIdentity` lives in the forge crate and `Writer`/`Config` in the forge client crate**, not the root, and `effect_key` is in the forge client's identity.rs. Cosmetic.
14. **The `Error` variants are in a different order from forge.md 3.4's table.** Cosmetic, unless a store codec numbers them by order; temper-protocol-store should then fix the numbering explicitly.
