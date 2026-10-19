# Session 09: the protocol world

Move five fake-protocol tests into skein, then join the client and component under a scripted durable
owner. Prove recovery, listings, signed deliveries and the three protocol invariants. Needs at start:
skein 08.1's Joint/fixtures, skein 07.1's client-world LIMITS/translation, temper 03.1's users on the
component, temper 05.1's users on the fakes and skein 06.2's transcripts/deliveries, landed on origin/main.

## Read first

- skein `docs/design/forge-connection.md` 4.1, 6–8, 10; `forge-client.md` 2.1, 8, 10; `fake-forge.md` 4, 8; `forge.md`
  3.7, 5.4.
- skein `docs/foundation/programming-model.md` 4.2–4.4, 5.2–5.4, 6.3, 7, 9, 10.2; `testing-strategy.md`
  2.4–2.5, 3, 4.4, 6–8, 10.
- README, code-organization.md 1, 3, 5–8, decisions.md 11, 18, 22, 23, 25 and Design questions36; later.md;
  findings/{survey-03-connection,survey-06-tests,trace-connection,
  trace-fakes,base-skein-03,base-skein-07,base-skein-08}.md. The last is this session's checked inventory;
  earlier drafts' 32-test partition is obsolete.

## What the design asks

- Connection 6, 8: authenticated hints after responses; each phase's deadline; Close drops outstanding work,
  then closed once and last after io settles.
- Connection 10: client/component stories and invariants over the fake's bytes; skein08 owns the
  machine world's plaintext/TLS byte faults and independently metered worst case.
- Fake 4, 8: its existing HTTP/domain boundary, observations and tally; testing-strategy 8: preserve moved
  coverage and understand slow tests.

## Choices and rules that bind it

Copy temper's origin/main at each move's start, naming the source commit. Files and counts below are **at
the base**; preserve additional tests landed before copying. No rewrite of a moved part; only the
home-forced changes listed in Shape. Count file pairs by code-organization 6. A moved file over 800 lines is
split by its next changing increment, never by the move. No `temper` in skein code, comments, fixtures or
values; use `kit`, `kit/0.0.0`, `b"kit"`; no serde, alias or re-export of a moved type. Each world is a hand-listed
Cargo.toml member. No optimized test builds, raised budgets or filters hiding moved tests. New units <20 ms,
focused world tests <0.5 s; new focused work aims at ≤1 s per world, fuzzy ≤5 s. Decision 11 preserves the
four inherited 7.481–7.709 s sweeps in 08.1. All Sizes here include tests. 09.4's cost invariant is answer 36's (decisions.md, Design
questions 36; findings: "S08-cost"): a request cut inside its head is charged and not served.

## Increments

### 09.1 The five movable fake-protocol tests
Scope: decision 22's component/fake portion, at tests/forge/protocol.
Needs: the session's start capabilities, including skein 08.1's Joint/fixtures and skein 07.1's
  translate::commit (used by both job tests).
Files: temper tests/engine/protocol-forge/tests/fake_protocol.rs -> skein
  tests/forge/protocol/tests/fake_protocol.rs; new Cargo.toml/src/lib.rs; skein Cargo.toml (explicit
  member/dependencies), Cargo.lock. Manifest dependencies: skein-forge, skein-lib,
  skein-forge-machine-world (path ../machine), skein-forge-client-world (path ../client),
  skein-fake-forge-domain (raw operations for direct), skein-fake-forge-protocol (Route).
Shape: set `license.workspace = true` in the new Cargo.toml's `[package]`, beside its other
  workspace-inherited fields (README, Workspace license inheritance).
  Package skein-forge-protocol-world; import Joint directly from skein_forge_machine_world::joint, api
  from skein-forge, and commit from skein_forge_client_world::translate. No copied driver or translation.
  Retain Joint::new/failing(seed: u64) -> Self, call(&mut self,u64,Repository,Op), answered(&self,u64) ->
  Option<&Answered>, skein_forge_machine_world::joint::REPO ({forge:1,repository:2}),
  and Joint::direct (BUILD_STEP_NAME/BUILD_STEP_LOG stay private consts of the moved fake_protocol.rs); never fixtures::REPO or the client world's REPO.
Test shape: 5 moved tests, names unchanged, no new test:
  the_component_starts_and_lists_items_through_the_fakes_bytes;
  pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one;
  opening_with_nothing_to_merge_writes_no_pull_request;
  a_failed_job_read_names_its_failed_step_and_reads_its_log;
  a_failed_jobs_log_end_comes_through_a_suffix_range. Preserve fixture limits and assertions: first list
  cost2/1 item/4 requests; lookup cost4; empty pull NothingToMerge/cost4/no write; jobs cost3, step1/"Run
  build", suffix b"build 3\n". Recorded base rerun: 5/0.010 s (findings/base-skein-08.md).
Proof:
joint; `heavy cargo nextest run -p skein-forge-protocol-world`: 5 passed, each <0.5 s; base form --test fake_protocol, 5/0.010 s lighter-load rerun. This proves the moved HTTP/domain joint; no socket behavior changes.
Don't: move or delete these 10 temper-policy tests:
```text
a_change_produced_opened_checked_queued_and_landed
a_created_issue_with_its_answer_lost_is_found_after_restart
a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice
a_failed_ci_head_is_repaired_then_checked_again
an_external_move_of_a_held_branch_is_reported_as_drift
idle_forge_call_cost_does_not_follow_old_issue_history
honest_costs_equal_the_requests_the_fake_served
a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one
the_api_token_reaches_no_record_request_or_fact
stories_through_cut_streams_answer_each_call_once
```
  Temper retains protocol.rs, writer_settings/protocol_shared/goal_view and its owner-valued
  settings/shared/token_absent fixtures, plus engine-forge's LIMITS/fake_config for its other worlds. It
  uses skein's Pipe/Joint and shared fake_limits; temper 07 deletes only moved files/tests and repairs
  imports/manifests, preserving protocol.rs's access to 08.1's public seams.
Size: about 113 lines moved, 40 new including tests: 30 manifest/module lines, 10 import/path edits.
Removes: nothing in temper; its owner-policy world remains after temper 07.

### 09.2 The owner, token maps and restart
Scope: forge-connection 10's protocol world; forge-client 8, 10's durable owner.
Needs: 09.1's crate; skein 02.1's client, 03.1's component, 05.1's domain and 05.2's protocol layer (already
  landed); 07.1's LIMITS/translation.
Files: tests/forge/protocol/src/{lib,world,fixtures}.rs (new modules), tests/owner.rs (new), Cargo.toml
  (add skein-forge-client and skein-forge-connection dependencies), Cargo.lock;
  tests/forge/machine/src/joint.rs (test-only response-cut hook).
Shape: use client::Domain and Joint, never the source's connector Protocol. The owner keeps Map<Key,Stored>,
  Map<u64,Entry>, writer claims and held entries under client limits; records Save/Erase/Progress/Outcome as
  one durable decision before releasing Call. Map each component token to a client token and generation;
  route Answered and authenticated Hint. New World owns those tables, queues, client Env, Joint and logs;
  all fields private. Every operation names skein_forge_machine_world::joint::REPO ({1,2}).
  The sole fixtures module uses 07.1's LIMITS; mirror its
  calls/rows/answer_bytes/op_bytes/lifetime into Shared and set component/shared repositories to
  LIMITS.repositories (1); other 08.1 caps stay. Component::new admits this
  combination in the scratch probe; the fake keeps its existing admitted limits. The kit has no shipped application profile. Keep the fake's existing page
  cap 2 and zero late/lost hooks, with kit owner values. The loop drains ready work and advances to the next
  client/component/fake deadline, bounded turns; no sleeps or reads of client internals. Restart keeps the
  fake and durable owner, drains old terminals without delivery to the new generation, replaces
  component/client, Restore then Restored, Writer claims, PauseOutbox, non-held Make, ReadAfresh, then
  SettleOutbox on ReadAfreshDone. No attempt/deadline is renewed.
```rust
// tests/forge/protocol/src/world.rs; client is skein_forge_client.
/// Scripted durable owner; finish settles its accepted calls.
pub struct World { /* private owner, client, Joint, observation logs */ }
impl World {
    pub fn new(seed: u64, cut_chance: u32) -> Self;
    pub fn event(&mut self, event: client::Event);
    pub fn run_for(&mut self, span: skein_lib::Duration);
    pub fn restart(&mut self);
    pub fn finish(&mut self);
    pub fn creations(&self) -> u32;
    pub fn made(&self, entry: u64) -> bool;
    /// Writes without committed Progress; old-generation answers delivered to the new client.
    pub fn owner_counts(&self) -> (u64,u64);
}
// tests/forge/machine/src/joint.rs; impl Joint
/// Cut after the fake commits a creation, before its first response byte.
pub fn lose_next_creation_answer(&mut self);
// tests/forge/protocol/src/fixtures.rs
pub fn lost_creation(seed: u64) -> World;
```
  The two counts come from independently retained decision/release and token-delivery logs, not client
  internals. New additive hooks preserve all existing signatures. Temper's side: after temper07,
  tests/engine/protocol-forge/src/protocol.rs uses Joint::{call,tick,time,replace,busy}, its public
  component/env/above/below/fake_env/answers, and joint::checks; tests/protocol.rs and fuzzy_protocol.rs
  drive that owner. Its next repin retains those calls, with loss injection off by default; no call breaks.
Test shape: two new focused tests in tests/owner.rs (<0.5 s each), no moved tests. Reuse fixtures::lost_creation
  (entry1/task1, CreateIssue, title/body b"kit", empty labels, no condition/start/attempt, failures0,
  repository skein_forge_machine_world::joint::REPO). Stop at the first lost-response terminal before
  resuming the client, so the fake has Opened once and the durable Progress has an attempt.
  a_decision_is_committed_before_its_call: saved attempts1 before released creations1, writes without
  committed Progress0, no Made yet. Reference (also check that nonempty decision/release log):
```rust
// tests/forge/protocol/tests/owner.rs
#[test]
fn a_decision_is_committed_before_its_call() {
    let world = fixtures::lost_creation(23);
    assert_eq!(world.creations(), 1);
    assert!(!world.made(1));
    assert_eq!(world.owner_counts(), (0, 0));
}
```
  old_generation_answers_are_discarded: keep one already accepted old-generation component terminal
  in the owner's delivery queue, restart, then drain it; old answers delivered to new client0, new
  generation1, original attempt and expiry unchanged. Finish: creations1, Made(entry1)=true; owner_counts
  (0,0). Use a test-local fixture variant of lost_creation to hold that one delivery, never inspect client
  private state. These support tests exercise both logs and restart; 09.3 supplies the public stories.
Proof:
joint; `heavy cargo nextest run -p skein-forge-protocol-world`: 7 passed (5+2), each <0.5 s, new focused sum ≤1 s; the scripted owner exposes its commit/restart boundary, which conformance cannot.
Don't: copy temper's Protocol/policy, bypass HTTP with translate::answer, renew an attempt/deadline,
  or duplicate the client world's LIMITS/translation.
Size: about 0 lines moved, 300 new including tests: 150 decision pump/durable tables, 70 token/restart
  routing, 30 fixture/loss hook, 50 support tests/log assertions; no shipped code.
Removes: nothing.

### 09.3 The protocol stories
Scope: forge-connection 10's creation recovery, listings and signed deliveries under 09.2's owner.
Needs: 09.2's durable owner, token maps and restart; its same client/component/fake capabilities.
Files: tests/forge/protocol/src/{world,fixtures}.rs, new tests/protocol.rs;
  tests/forge/machine/src/joint.rs for signed-delivery observation, using its existing public hints.
Shape: extend the existing owner fixture; no second Joint, driver or translation. Limits remain 09.2's:
  07.1's LIMITS mirrored into Shared, repositories1, fake page cap2, no late/lost hooks. Observe the
  signed delivery before it enters the component and the 204 queueing before Hint, not a direct answer.
  Every story uses skein_forge_machine_world::joint::REPO ({forge:1,repository:2}); the fake holds id2.
```rust
// tests/forge/machine/src/joint.rs; impl Joint, additive getters, #[must_use].
/// Signed requests retained before they enter the component, for independent HMAC verification.
pub fn deliveries(&self) -> &[Vec<u8>];
/// Queued 204s, offered hints, and whether every hint followed its queued response.
pub fn delivery_counts(&self) -> (u64,u64,bool);
```
  Temper's side: 09.2's retained protocol.rs callers keep their signatures and behavior; no repair
  besides taking the new pin. The one-delivery story checks delivery_counts()==(1,1,true).
Test shape: 3 new tests in tests/protocol.rs, each <0.5 s; no tests move.
  a_creation_lost_on_the_wire_is_found_after_restart: entry1/task1/skein_forge_machine_world::joint::REPO, CreateIssue title/body b"kit",
  empty labels, no condition, no start/attempt, failures0; arm the Joint hook, make, and stop at the first
  lost-response terminal before resuming the client. The fake observes one Opened; owner retains the durable
  Progress/attempt, no Made.
```rust
#[test]
fn a_creation_lost_on_the_wire_is_found_after_restart() {
    let mut world = fixtures::lost_creation(23);
    assert_eq!(world.creations(), 1);
    assert!(!world.made(1));
    world.restart();
    world.run_for(skein_lib::Duration::from_secs(30));
    assert_eq!(world.creations(), 1);
    assert!(world.made(1));
}
```
  listings_follow_another_partys_changes: owner reads Items through client; person creates A/B/C across
  cap2, then edits B and closes C; fresh listing equals the independent fake snapshot (3 ids, B's new title,
  C Closed), no duplicate ids across pages; no direct fake answer fed to the client.
  signed_deliveries_reach_the_clients_owner: hooked Joint, person creates one issue; independently verify
  captured delivery's HMAC, one 204 queued before one Hint, and the client's ensuing read observes that
  issue.
Proof:
joint; `heavy cargo nextest run -p skein-forge-protocol-world`: 10 passed (7+3), new focused sum (09.2+09.3) ≤1 s, each <0.5 s. Base form has 5 movable tests; new stories first run here. This is the highest layer observing the scripted owner's durable commit/restart boundary; conformance has no client owner.
Don't: copy temper's Protocol/policy, bypass HTTP with translate::answer, inspect client private state, or
  implement late/reordered/lost deliveries and the with/without-hint variants deferred in later.md.
Size: about 0 lines moved, 270 new including tests: 140 story/observation support, 40 fixtures,
  90 three story tests; no shipped code.
Removes: nothing.

### 09.4 The three protocol invariants
Scope: forge-connection 10's call, cost and credential invariants.
Needs: 09.3's owner/client/component joint and its same 02.1/03.1/05.1/05.2 capabilities; its cost
  rule is answer 36's (decisions.md, Design questions 36).
Files: tests/forge/protocol/src/{world,fixtures,referee}.rs (referee new); tests/fuzzy_protocol.rs (new);
  tests/forge/machine/src/{joint,pipe}.rs for observation.
Shape: independent ledger of submitted tokens, first-byte sends, complete request heads and component
  terminals; observe raw boundaries, never infer expected cost from Answered.cost. Keep start traffic
  separate across restarts; finish stops new client calls and settles the submitted set before owner Close
  (Close deliberately emits no terminal, 08.3). Scan records, owner events/facts, request METHOD/target
  lines and frozen fake observations for token and hook-secret bytes, textual and Debug-byte forms; do not
  scan authorized header/body transport as forbidden data. New World methods:
  calls_and_answers(&self)->(u64,u64), accounting(&self)->(u64,u64,u64,u64) (reported cost; requests
  whose first byte the component wrote, counted at the world's streams; requests the fake served;
  requests whose stream was cut before the head's blank line; starts excluded from each);
  secrets_absent(&self)->bool. Add these observer methods (all getters #[must_use]):
```rust
// tests/forge/machine/src/pipe.rs; impl Pipe
/// Written requests and requests cut before the head blank line; includes startup traffic.
pub fn request_counts(&self) -> (u64,u64);
/// Only METHOD/target lines, including partial lines; never authorization headers or bodies.
pub fn request_lines(&self) -> &[Vec<u8>];
// tests/forge/machine/src/joint.rs; impl Joint
/// Stream counts summed across closed pipes, starts excluded: written and cut inside a head.
pub fn request_counts(&self) -> (u64,u64);
/// Request lines retained across closed pipes; the owner scans them for secrets.
pub fn request_lines(&self) -> &[Vec<u8>];
```
  Count first-byte writes at Pipe::client_down's Down::Send; detect delivered complete heads by their
  blank line at the server side, not the fake tally or Answered.cost. Track body lengths so body bytes
  cannot begin another request. Save observations before Joint::end removes a pipe, across restart;
  observer code draws no RNG and changes no cut schedule. Tag start exchanges separately, including
  partially written starts; the mixed fixture completes start before releasing client calls each generation.
  Temper's side: retained protocol.rs's Joint::{call,tick,time,replace,busy,tally} and public fields,
  tests/{protocol,fuzzy_protocol}.rs, and setup's exchange.rs Pipe::{new,client_down,server_down,
  client_up,server_up} keep their signatures (Pipe::has_work's one user is Joint). Additive observers require no call-site repair at
  its next repin. Preserve their behavior and the setup world's Pipe use after skein07.3/temper07.
  fixtures::mixed_calls(seed: u64)->World scripts bounded reads/writes, random cuts (300/1000), restarts and
  final settlement; caps from 09.2.
Test shape: exactly 3 new fuzzy tests, seeds 0..16 and pinned 26; the three's sum ≤5 s, same
  unoptimized fixture: each_call_has_one_answer_under_stream_cuts: unique-token set equals terminal set
  after finish, each multiplicity 1, outstanding 0. reported_costs_account_for_served_and_cut_requests: in
  every run, reported = written, and served = written less those cut inside a head (answer 36). A call
  holds one request at a time, so a cut ends at most one. Seed 26 (base: cost 2, served 1, starts 2,
  Timeout) passes only with 2 written and 1 cut inside a head; any other count there is a fault. Cuts
  inside a head stay in the mix; do not weaken or hide cuts.
  Retain the seed26 calibration as a Joint::cutting(26,300,zero-faults) subcase: run20 s for start,
  call1/skein_forge_machine_world::joint::REPO/Items sinceTime::ZERO page1 kindNone, then run11 s. The mixed owner may consume RNG
  differently, so its seed26 alone is not a claim to reproduce that base request trace.
  no_credential_reaches_a_record_request_line_or_fact: no matches for either secret in
  all retained values, including refusal/restart paths; assert nonempty records, request lines and owner
  facts so the check cannot be vacuous. Reference for tests/fuzzy_protocol.rs (the other two reuse its
  fixture):
```rust
#[test]
fn each_call_has_one_answer_under_stream_cuts() {
    for seed in (0..16).chain([26]) {
        let mut world = fixtures::mixed_calls(seed);
        world.finish();
        let (calls, answers) = world.calls_and_answers();
        assert!(calls > 0, "seed {seed}");
        assert_eq!(answers, calls, "seed {seed}");
    }
}
```
  The cost test asserts (reported,written)==(written,written) and
  served==written.checked_sub(cut_inside_head).expect("cuts are written requests") in every run;
  its calibration asserts (reported,written,served,cut_inside_head)==(2,2,1,1), starts2 and Error::Timeout.
  The call referee also asserts outstanding0 and each token multiplicity1; the credential test asserts
  secrets_absent()==true, nonempty records/request lines/owner facts, and secret matches0.
Proof:
joint; `heavy cargo nextest run -p skein-forge-protocol-world --profile fuzzy`: 3 passed, ≤5 s total. Base source fuzzy run is 6/7.709 s (rerun 1.895 s); it checks source viability, not these absent client-only tests. The joint referee is the highest observer of the invariant; no process proof sees it.
Don't: count startup as a client call, count fake domain calls as HTTP requests, copy temper's invariant
  tests, or count a request cut inside its head as served.
Size: about 0 lines moved, 300 new including tests: 110 stream/Joint observers, 100 referee/fixture,
  90 invariant tests/calibration; no shipped code.
Removes: nothing.

## Questions to raise when reached

A new sweep over its time aim: report the failing case, time and cause, never raise a budget. Decision18
allows +1.5 s default/+6 s fuzzy for the whole plan; 08+09's new bounds are 2 s/10 s, exceeding them by
0.5 s/4 s before moves. Report actual sums alongside 08's inherited max7.709 s/summed30.369 s four sweeps
and README's whole-suite measurements. Answer36 is settled, not a pending design question.

## Done when

From skein; base commands/results are findings/base-skein-08.md. Record the new tests' elapsed times
from these commands and sum the five focused tests (09.2/09.3) and three fuzzy tests (09.4), separately.
Report with 08's sums against decision18's +1.5 s/+6 s whole-plan aims, with moved times separately;
report any excess with the three slowest tests. Summed test seconds are conservative accounting,
not whole-suite wall time: README's three gate runs still decide the whole-run aim.

- `heavy cargo nextest run -p skein-forge-protocol-world` prints 10 passed, new focused sum ≤1 s, each <0.5 s.
- `heavy cargo nextest run -p skein-forge-protocol-world --profile fuzzy` prints 3 passed, sum ≤5 s.
- `heavy cargo clippy -p skein-forge-protocol-world --all-targets -- -D warnings` prints Finished without warnings.
- `heavy rg -n '^    "tests/forge/protocol",$' Cargo.toml` prints exactly one member line.
- `heavy rg -n -i temper tests/forge/protocol` prints nothing (rg exit1); README Done when5 covers the whole kit.
