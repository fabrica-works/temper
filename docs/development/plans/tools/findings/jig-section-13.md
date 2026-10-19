# What jig and skein have for client/tools.md, section 13

Date: 2026-10-18. Facts for the tools plan (development.md, 5.4 "a code audit"); no plan. Bases: skein 49e7eb6 and jig 08430b94 (both `origin/main`, the primary checkouts), temper's worktree at 4735d1852. Pins in temper's `Cargo.lock`: jig 08430b94, skein 8a51e10, smith 29fa97c9; jig's own lock pins skein 0f093bb. The graph was not used (jig is not indexed, temper's index is stale): every name was checked with `rg` in the checkout. Commands run from `~/src/rust`, `T=tmp/temper/tools-plan`; in listings `temper:` stands for `$T/` and `{a,b}` folds paths. Code is named by path and symbol. "not verified" marks what no command here shows.

## 1. A named key
### 1.1 skein's request table (`skein/crates/skein-lib/src/requests.rs`)
- `RequestTable<O, R>` has `new(limits, seed)`, `ask(owner, scope, size, request) -> Result<RequestKey, R>` and `restore(owner, size, record) -> Result<(), RequestRecord<R>>`. No other method admits a request.
- `ask` draws the key with the private `fresh_key`: 8 bytes of `namespace` (the seed) and 8 of SplitMix64 from `namespace` and `draws`, retried past held keys at most `limits.requests + 1` times. The caller cannot name it.
- `ask` returns `Err(request)` for four causes the caller cannot tell apart: `!fits(size)`, `serial == u64::MAX`, no `fresh_attempt`, no `fresh_key`.
- `restore` keeps `record.key` and refuses a held one (`self.contains(record.key)`), returning the record. `RequestRecord { key, scope, first_sent, request }` has `pub` fields, so a caller can build one.
- `restore` inserts with `restored: true`: `save: !restored` makes the entry emit no first record (`Save`), and it stays parked until `retention` is known. `ask` inserts with `restored: false`: it saves first and is ready once the link is up and its scope confirmed.
- `lib.md`, section 12, "Keys": "each admitted request gets a key drawn from the seed ... A restored request keeps its key." It states no admission under a given key. Since both pins only 78dd8fa ("Hash lib containers by their held state") changed `requests.rs`.
```
$ rg --no-line-number 'pub fn (ask|restore)|fn fresh_key|return Err\((request|record)\)|self.contains\(record.key\)' skein/crates/skein-lib/src/requests.rs
    pub fn ask(&mut self, owner: O, scope: u64, size: u64, request: R) -> Result<RequestKey, R> {
            return Err(request);
            return Err(request);
    pub fn restore(&mut self, owner: O, size: u64, record: RequestRecord<R>) -> Result<(), RequestRecord<R>> {
        if !self.fits(size) || self.serial == u64::MAX || self.contains(record.key) {
            return Err(record);
    fn fresh_key(&self) -> Option<(RequestKey, u64)> {
$ rg --no-line-number 'fn insert|let eligible|save: ' skein/crates/skein-lib/src/requests.rs
    fn insert(&mut self, owner: O, size: u64, record: RequestRecord<R>, restored: bool) {
        let eligible = self.up && self.scope == Some(record.scope) && (!restored || self.retention.is_some());
            save: !restored,
$ git -C skein log --oneline 0f093bb..origin/main -- crates/skein-lib/src/requests.rs
78dd8fa Hash lib containers by their held state
```
### 1.2 jig-client-domain (`jig/crates/jig-client-domain/src/`)
- The table is `RequestTable<crate::Key, crate::Held>` (`domain.rs`). Its owner value is the application's `Key { asker: Asker, number: u64 }` (`boundary.rs`); the engine's key is the `RequestKey` (`[u8; 16]`) the table draws inside `ask`, scoped to the party, sent in `Down::Send { attempt, key: RequestKey, request }`.
- `Event::Send { key: Key, request: Ask }` (`boundary.rs`; the same in `jig-client/src/client.rs`) carries the application key only; no event or `Ask` variant carries an engine key.
- `Domain::send` (`requests.rs`): a held application key with the same `Ask` returns at once (joined, nothing told); with another `Ask` it panics, `assert!(held.request == request, "request key conflict")`, and `tests/requests.rs` has a `should_panic` test for it. Without a signed-in identity it tells `Outcome::Unconfirmed`. `self.table.ask(key, party, size, held)` discards the drawn `RequestKey`, and any `Err(_)` tells `Outcome::Limit`.
- `restore_record` calls `self.table.restore(...)`; an invalid record, a key already in `self.requests` or a refusal is counted (`Told::Dropped { count }`), the rest are `Told::Kept`. `not_kept(key)` (the store did not write the record) tells `Outcome::NotKept` for the owner `self.table.not_kept(key)` hands back.
- `client/domain/client.md`, section 6: "A second `Send` under a key the client holds is joined to the first when it carries the same request ... with another request, it is a key conflict, a mistake of the application's, and loud ... The engine knows the request by the key the table draws, scoped to the party (contract.md, 3.1)". `contract.md`, 3.1: "One key, one ask: a key sent again with a different ask is refused as a conflict."
- `Refusal::KeyConflict` (`jig-domain-people`; `contract/refusals.rs`) is the engine's refusal for one engine key with another ask, not the client's panic.
- The boundary `Outcome` has five variants: `NotKept`, `Answered(Answered)`, `Unknown`, `Limit`, `Unconfirmed`. `contract::Outcome` (`contract/asks.rs`), `EscalationOutcome` and temper's wire and forge outcomes are other enums.
```
$ rg --no-line-number 'request key conflict|self\.table\.ask|self\.table\.restore|Outcome::(Unconfirmed|Limit|NotKept)' jig/crates/jig-client-domain/src/requests.rs
            assert!(held.request == request, "request key conflict");
            tell(out, Told::Answered { key, outcome: Outcome::Unconfirmed });
        match self.table.ask(key, party, size, held) {
            Err(_) => tell(out, Told::Answered { key, outcome: Outcome::Limit }),
                || self.table.restore(key, declared(&env.limits, &request), record).is_err()
            tell(out, Told::Answered { key: owner, outcome: Outcome::NotKept });
$ rg --no-line-number 'Send \{|pub struct Key|pub enum Outcome' jig/crates/jig-client-domain/src/boundary.rs jig/crates/jig-client/src/client.rs
jig/crates/jig-client-domain/src/boundary.rs:    Send { key: Key, request: Ask },
jig/crates/jig-client-domain/src/boundary.rs:    Send { attempt: Token, key: RequestKey, request: Ask },
jig/crates/jig-client-domain/src/boundary.rs:pub struct Key {
jig/crates/jig-client-domain/src/boundary.rs:pub enum Outcome {
jig/crates/jig-client/src/client.rs:    Send { key: Key, request: Ask },
$ rg --no-line-number -A1 'should_panic' jig/crates/jig-client-domain/src/tests/requests.rs
#[should_panic(expected = "request key conflict")]
fn a_second_send_with_another_request_is_a_loud_key_conflict() {
```
Not verified: what the small store holds for a request admitted by `restore` (it emits no `Save`), and how `limits.requests` counts it.

## 2. A read of one turn
### 2.1 The engine side has it
- `jig-domain-core/src/reads/vocabulary.rs`: `Read::Turn { task: u64, attempt: u64, turn: u32 }`, `Scope::Turn` of the same fields, `Past::Turn(TurnRecord)`; `reads.rs` maps `Read::Turn` to `Scope::Turn`. No non-test line builds a core `Read::Turn`; `tests/core/src/referee/offers.rs` and `tests/store/src/world.rs` name it (how: not verified). `protocol/store.md`, 7.3: "a turn | the task's transcript (8.3)".
- `jig-protocol-store/src/reads.rs`: `Scope::Turn` goes to `point::turn`, which asks the transcripts for `TranscriptRead::Turn { task, attempt, turn }` with `bytes: range.bytes`. In `reads/point.rs`, `finish` returns `Err(Failure::Cut)` when `row.owned_bytes() > party.bytes` (a spent join budget gives `Cut` too); the store's `answer` turns `Err(Failure::Cut)` into `Event::Read { owner, result: empty() }`, and `empty()` is `Loaded::Page { rows: [], next: None }`.
- `jig-domain-core/src/reads/rows.rs`, `from_store`: a stored row with `total > range.bytes` answers `Answer::None(Why::Busy)`. `is_page` is false for `Scope::Turn`, so with no usable row the function ends in `Answer::None(Why::Unknown)`; a turn past the byte bound thus reaches the core as an empty page, is answered `Why::Unknown` (not `Busy`), and carries no size. Fn `object` answers a matching `Scope::Turn` row with `historical(Past::Turn(value), Some(value.at), None)`. Whether `row_bytes` and `owned_bytes` can disagree so that the `Busy` line is reachable for a turn: not verified.
```
$ rg --no-line-number 'Turn \{ task' jig/crates/jig-domain-core/src/reads.rs jig/crates/jig-domain-core/src/reads/vocabulary.rs
jig/crates/jig-domain-core/src/reads/vocabulary.rs:    Turn { task: u64, attempt: u64, turn: u32 },
jig/crates/jig-domain-core/src/reads/vocabulary.rs:    Turn { task: u64, attempt: u64, turn: u32 },
jig/crates/jig-domain-core/src/reads.rs:        Read::Turn { task, attempt, turn } => Some(Scope::Turn { task: *task, attempt: *attempt, turn: *turn }),
$ rg --no-line-number 'Failure::Cut|fn empty|Scope::Turn \{ \.\. \} =>|point::turn' jig/crates/jig-protocol-store/src/reads.rs jig/crates/jig-protocol-store/src/reads/point.rs
jig/crates/jig-protocol-store/src/reads/point.rs:            spent.joins = spent.joins.checked_sub(1).ok_or(Failure::Cut)?;
jig/crates/jig-protocol-store/src/reads/point.rs:        return Err(Failure::Cut);
jig/crates/jig-protocol-store/src/reads.rs:pub(crate) fn empty() -> Loaded {
jig/crates/jig-protocol-store/src/reads.rs:        Scope::Turn { .. } => {
jig/crates/jig-protocol-store/src/reads.rs:            let result = point::turn(store, owner, &range, above, below);
jig/crates/jig-protocol-store/src/reads.rs:        Err(Failure::Cut) => above.push(Event::Read { owner, result: empty() }),
$ rg --no-line-number 'total > range.bytes|^        Answer::None\(Why::(Unknown|Gone)\)' jig/crates/jig-domain-core/src/reads/rows.rs
        if total > range.bytes || row.ancestors.len() > usize::try_from(limits.tasks.depth).expect("depth fits") {
        Answer::None(Why::Gone)
        Answer::None(Why::Unknown)
```
### 2.2 The client side does not
- Wire `jig-client-protocol-wire/schema/client-v8.schema` (`VERSION = 8` in `src/lib.rs`; generated `src/generated/v8.rs`; `golden/v8`): `enum Read` has no turn. `ReadNone` is `unknown, refused, gone, busy, signed_out`: no size. `TurnTooLarge { bytes: u64 }` exists only in `TurnBytes`, inside `ArchivedTurn` of an activity line. A drift test: not verified.
- `jig-protocol-engine-clients/src/translate/`: `reads.rs` has `w::Read::Deployment => reads::Read::Deployment` and no turn arm; `pages.rs` lists `reads::Read::Turn { .. }` twice among arms that `return Err(invalid)`; `objects.rs` rejects `reads::Past::Reply(_) | reads::Past::Turn(_)` twice with `Reason::Tag`; `cursor.rs` encodes `Scope::Turn` (tag 7); `activity.rs` builds `TurnTooLarge { bytes }` for activity lines.
- `jig-client-domain/src/contract/`: `reads.rs` `Read` has 16 variants (Tasks to Deployment), `ReadValue` the `Owner*` documents and `Historical`; the word "turn" is absent from it. `turn.rs`: `TurnRead { Decoded(Turn), Undecodable { why: TurnProblem } }`, `TurnProblem { Version { found: u16 }, Malformed, TooLarge }`: `TooLarge` has no size.
- `jig-client-protocol-link/src/translate/`: `reads.rs` has `c::Read::Deployment => w::Read::Deployment` and no turn; `pages_part0.rs` maps `w::TurnBytes::TooLarge(_)` to `TurnProblem::TooLarge`, dropping `bytes`; `pages.rs` maps the decoder's `Undecodable::TooLarge` to it too.
- `client/domain/contract.md`, section 6, "Reads": a table with no turn row (its `activity{n}` row says "words, run boundaries, turns, what it heard"). Section 5.1 says a turn in an activity line is decoded "with no read of its own", and one past the page's bound "comes with its size instead, and does not decode, as too large". client/tools.md, section 13, cites `contract.md`, section 6, for `turn{n, attempt, turn}`; that section does not name it.
```
$ rg --no-line-number '^enum Read\b|^enum ReadNone|^record TurnTooLarge|^record ArchivedTurn' jig/crates/jig-client-protocol-wire/schema/client-v8.schema
enum Read { tasks: ReadTasks, me, task: ReadTask, proposal: ReadProposal, escalation: ReadEscalation, question: ReadQuestion, person_task: ReadPersonTask, result: ReadResult, ended: ReadEnded, history: ReadHistory, activity: ReadActivity, inbox: ReadInbox, spend: ReadSpend, project: ReadProject, notes: ReadNotes, deployment }
enum ReadNone { unknown, refused, gone, busy, signed_out }
record TurnTooLarge { bytes: u64 }
record ArchivedTurn { task: u64 name: TurnName cumulative: u64 read: option u64 at: u64 turn: TurnBytes }
$ rg --no-line-number 'Past::(Reply|Turn)|Read::Turn' jig/crates/jig-protocol-engine-clients/src/translate/pages.rs jig/crates/jig-protocol-engine-clients/src/translate/objects.rs
jig/crates/jig-protocol-engine-clients/src/translate/pages.rs:        | reads::Read::Turn { .. }
jig/crates/jig-protocol-engine-clients/src/translate/pages.rs:                    | reads::Read::Turn { .. }
jig/crates/jig-protocol-engine-clients/src/translate/objects.rs:        reads::Past::Reply(_) | reads::Past::Turn(_) => {
jig/crates/jig-protocol-engine-clients/src/translate/objects.rs:        reads::Past::Reply(_) | reads::Past::Turn(_) => {
$ rg -c -i 'turn' jig/crates/jig-client-domain/src/contract/reads.rs || echo none
none
$ rg --no-line-number 'TurnBytes::TooLarge|Undecodable::TooLarge\) =>' jig/crates/jig-client-protocol-link/src/translate/pages.rs jig/crates/jig-client-protocol-link/src/translate/pages_part0.rs
jig/crates/jig-client-protocol-link/src/translate/pages_part0.rs:            w::TurnBytes::TooLarge(_) => c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge },
jig/crates/jig-client-protocol-link/src/translate/pages.rs:        Err(jig_protocol_transcript::Undecodable::TooLarge) => c::TurnProblem::TooLarge,
```

## 3. Who names these enums
`wildcard_enum_match_arm = "deny"` is in the workspace lints of jig and of temper (`Cargo.toml`), so a match over these enums is exhaustive; `jig-client-domain` and `temper-client-view` inherit them (`[lints] workspace = true`), 27 of jig's 29 `crates/*/Cargo.toml` have that line, and the other crates: not verified. Each file below names a variant in an arm (matches), except where marked.
```
$ rg -l 'Read::Deployment' jig/crates/jig-client-view/src jig/crates/jig-client-protocol-link/src jig/testing $T/crates --glob '!**/tests/**' --glob '!tests.rs' --glob '!**/fixtures*'   # contract::Read
jig/crates/jig-client-view/src/{application,objects,page/inbox,page/routing,page/shown}.rs
jig/crates/jig-client-protocol-link/src/translate/reads.rs
jig/testing/jig-client-scripted-engine/src/{reads,watches}.rs
$ rg -l 'ReadValue::OwnerDeployment' jig/crates jig/testing $T/crates --glob '!**/tests/**' --glob '!tests.rs' --glob '!**/generated/**'   # contract::ReadValue
jig/crates/jig-client/src/facts.rs
jig/crates/jig-client-view/src/{application,objects,page/routing}.rs
jig/crates/jig-client-protocol-link/src/translate/reads.rs
jig/crates/jig-protocol-engine-clients/src/translate/objects.rs   (builds the wire's ReadValue)
jig/testing/jig-client-scripted-engine/src/{reads,requests,truth}.rs
jig/testing/jig-client-test-kit/src/check.rs
temper:crates/temper-client-domain/src/acts.rs
temper:crates/temper-client-view/src/{cards/proposal,pages/inbox}.rs
$ rg -l 'Outcome::NotKept' jig/crates jig/testing jig/examples $T/crates --glob '!**/tests/**' --glob '!tests.rs'   # boundary Outcome
jig/crates/jig-client-domain/src/requests.rs   (builds it)
jig/crates/jig-client-view/src/ui/client/outcome.rs
jig/examples/ops/client-domain/src/family.rs
jig/testing/jig-client-test-domain/src/family/client.rs
temper:crates/temper-client-domain/src/forge.rs
temper:crates/temper-client-view/src/{confirm,pages/chats}.rs
$ rg -l 'TurnProblem::(Version|Malformed|TooLarge)' jig/crates $T/crates --glob '!**/tests/**' --glob '!tests.rs'
jig/crates/jig-client-protocol-link/src/translate/{pages,pages_part0,pages_part1}.rs   (build it)
jig/crates/jig-client-view/src/turns/turn.rs
temper:crates/temper-client-view/src/pages/chat.rs
```
- temper has no match over `contract::Read`; its `ReadValue` matches are in `temper-client-domain` (`acts.rs`, also `pages/{chat,chats,inbox,task}.rs` by import) and `temper-client-view`.
- Tests naming these, not listed in full: `jig/tests/client/src/joint/{read_packets,read_ledger_pages,deployment_packets,scripted}.rs` and `joint/wire/*`, `jig/tests/ops/src/client/transport.rs`, `jig/tests/client-protocol/tests/client.rs`, `jig-client-view/src/tests/`, `temper:crates/temper-client-shell-terminal/src/tests/fixtures.rs`, `temper:crates/temper-client-view/src/tests/{chat,confirm}.rs`.

## 4. jig's `docs/design/STATUS.md`
- Row `client/domain/client.md` (partly) lists "1 to 4, 6 to 8, 10 to 12, 14 to 17, 19, 20" as covered, then excepts "17, 20: the service and one-shot modes, and the one-shot client's story: not planned (service and one-shot clients)".
- Bullet "Service and one-shot clients, a bot relaying decisions, and services' credentials": "the client's service and one-shot modes, with a service's root and the one-shot client's story ... Wait for an application whose services act as parties". `client.md`, section 20: "One-shot: sign in, read or send, wait for the durable answer, and stop, with the small store in a file". client/tools.md, section 13, lists the one-shot mode among what is "jig's as designed".
- In code the client has no mode: `jig-client/src/client.rs` has `With::Service(credential)` for a service's sign-in, and `jig-client-shell-native` and `jig-client-shell-web` exist; "one-shot" appears only in two doc comments of `jig-domain-core`.
- Bullet "smith's parity and wall-clock changes, for jig's next repin of smith": `smith-domain-tools` `Call` and `Outcome` variants for jobs, transcript version 5 (`NotRun::AfterFailedWrite`) in `jig-protocol-transcript`, event vocabulary 3, `Purpose::Ordinary { guard }`, the profiles' larger `memory`; "jig pins smith `29fa97c9`, before them. Waits for the plan that moves jig's pin of smith." base-repin.md says skein cannot move alone.
```
$ rg --no-line-number -i 'one.?shot|With::Service' jig/crates/jig-client/src jig/crates/jig-client-domain/src jig/crates/jig-domain-core/src --glob '!**/tests/**'
jig/crates/jig-client/src/client.rs:                    With::Service(credential) if credential.len() == 40 => Some(credential.clone()),
jig/crates/jig-domain-core/src/lib.rs:/// Authenticated one-shot reads whose answers wait for durability.
jig/crates/jig-domain-core/src/routing/vocabulary.rs:    /// An unkeyed one-shot read on the supplied live sign-in.
```
