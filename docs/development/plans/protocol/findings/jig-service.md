# Audit: jig-service

2026-10-14, read only, by a sub-agent of the planner, at temper `cc2697cf` with jig `d91a89c4`, smith `dfc0fe57` and skein `b5d3a2c3` (temper's pins; jig's, smith's and skein's paths are relative to their pinned checkouts). Evidence the plan cites (development.md, 4.1 and 5.4): where it and landed code differ, the landed code wins. The design gaps it found are closed by the Markdown landing "Design: close the gaps the protocol plan's code audit found"; `README.md` here maps each finding to its increment.

I've audited everything you asked about at jig d91a89c4 (smith dfc0fe5 and skein b5d3a2c where relevant). Most of what temper's design assumes is there, but nine places differ, collected at the end. Paths are relative to `~/.cargo/git/checkouts/jig-aba39083bf77fefe/d91a89c/`. I wrote no scratch files.

## 1. `testing/jig-test-service` (10,400 lines: about 7,675 source, 2,670 tests, Cargo.toml 55)

**Modules and line counts:** lib 543, startup 626, engine 453, route 277, stop 151, store 511, accounts 563, clients 558 + clients/translation 253, workers 319, worker 682, worker_route 266, limits 232, tools 420, family 234 + documents 183 + facts 494 + parts 706, system 127, values 42, boundary 35.
- Tests: fixtures 649 + fixtures/family 150, engine 510, clients 490, worker 325, first_period 177, stop 166, family 108, tests 74, startup 21.
- The store's application routing lives in another crate: `testing/jig-test-store/src/route.rs` (296 lines). Its keys (117) and records (422 + 464) are the test application's codecs.

**Public API**
- `lib.rs` exports: `pub mod family, limits, system`; `pub use {AccountsRuntime, Worker, WorkerRuntime, End, Files, PartyOutput, Seeds, Stop, Engine, Limits, named, startup}`.
- `pub struct Agent { endpoints: smith_protocol_channel::Endpoints, limits: smith_host_protocol::Limits, bounds: jig_domain_host::Bounds, host_limits: smith_host_domain::Limits }` (lib.rs:36-41).
- `pub fn named(document: &[u8]) -> Result<List<Box<[u8]>>, Problems>` (startup.rs:515).
- `pub fn startup(document: &[u8], files: &Files, agent: Option<&Agent>, seeds: &Seeds, accounts: Option<AccountsRuntime>, runtime: Option<WorkerRuntime>) -> Result<Service, Problems>` (startup.rs:533).
- `Files { entries: List<(Box<[u8]>, Secret)> }`, `Seeds { deployment: [u8;16], seed: u64, crypto: Secret }`, `End { Drained, Aborted, Stopped(Stop) }`, `Stop { CommitFailed{number}, Start(Problems), Listener }` (boundary.rs).
- `AccountsRuntime { root: Fd, effective_user, issuers: List<(u32, driver::Endpoint)>, trust_roots, trust_root_bytes }` (accounts.rs:19-26); `WorkerRuntime { roots: List<Fd> }` (worker.rs:17-19).
- `Service` methods: `adopt_store(root, user)`, `adopt_signals(fd)`, `completions/submissions/links/engine`, `end`, `worst_case`, `operations`, `next_deadline`, `next_policy_deadline`, `work_pending`, `is_empty`, `shutdown` (lib.rs:89-225).
- `pub fn iterate(service: &mut Service, now: Time, wall: Wall)` (lib.rs:393). It is the only public iterate. The engine's is `engine::iterate(&mut Engine, Time, Wall)` (engine.rs:279) and the worker's is `Worker::iterate(&mut self, Time, Wall)` (worker.rs:445); neither is exported.
- Engine layer: `Engine { domain: jig_test_domain::Domain, accounts, workers, clients, values, tools, store, systems: [System;2], first_periods, … }` (engine.rs:9-34).
- Worker layer: `Worker { root: jig_test_worker::Domain, link: jig_protocol_worker_link::Component, processes: Map<Token, smith_host_protocol::Process>, values: Table, application: Table, … }` (worker.rs:21-50).

**How io events are routed (`route_events`, lib.rs:492-543)**
- A `Shutdown` event calls `service.shutdown()`. The first call drains; the second sets `End::Aborted` and aborts the workers (lib.rs:204-225).
- Owner `Token(1)` is the signals source.
- Each worker is tried first through `worker.accepts`. A worker's owner is `(3<<60)+((i+1)<<48)`, plus 1 for its root close and 2 onward for its processes (worker.rs:594-599).
- Then accounts (`3<<60`, accounts.rs:17,502), then clients (`4<<60`, clients.rs:13,254).
- Anything else goes to the engine's workers component: listeners `1<<60`, connections `2<<60` (workers.rs:11-12).
- Store kernel ops use the top nibbles 8 to 10 (store.rs:384).
- Accounts and workers share nibble 3. Only the order of the checks keeps them apart.

**Where application-specific parts plug in**
- **Worker wire parts:** the type is `jig_protocol_worker_wire::Part { token: Token, bytes: Box<[u8]> }` (crates/jig-protocol-worker-wire/src/lib.rs:40-43).
  - Engine side: `kit::Event::Down { channel, down, parts: Box<[Part]>, values }` and `kit::Request::Up { channel, up, parts }` (crates/jig-protocol-engine-workers/src/boundary.rs:30-35,59).
  - Worker side: `link::Event::Up { up, parts }` and `link::Request::Down { down, parts }` (crates/jig-protocol-worker-link/src/boundary.rs:10,35-38).
  - The reference never fills or reads parts. It sends `parts: Box::default()` (workers.rs:308; worker.rs:329) and drops the ones it receives (workers.rs:243-245; worker.rs:277-279). These four sites are where temper's encoders and decoders go.
- **Store:** `jig_protocol_store::Write::Put { key, value } / Remove { key }` (crates/jig-protocol-store/src/boundary.rs:36-49) and `Restore::Application(Box<[skein_kv::Range]>)` ("a connector's live intervals", :53-60).
  - jig-test-store's route.rs translates them: `Put` at :61, `Remove` at :64, and `RestartStep::RestoreConnector{connector}` becomes `Restore::Application([live(connector)])` at :119. Rows come back up in `up()` (:182, application rows at :264-269).
  - The service delegates through `Store::down` → `route::down` (store.rs:143-144) and `route::up` (store.rs:451).
  - The format and sizes are handed in as `sizes::Application { version, key, value }` (jig-protocol-store sizes.rs:7) to `Store::on_directory(limits, application)` (jig-protocol-store lib.rs:93; store.rs:76).
- **Client application family:** families are declared as `kit::Served { family: Some(1), first/last: jig_test_protocol_client_wire::VERSION }` (clients.rs:105-111).
  - Inbound: `family::to_root(ToApplication, bound, rows)` (family.rs:127-189).
  - A request that does not decode, or of another family or version, gets `FromApplication::Undecodable{to}`.
  - A read that does not decode gets `FromApplication::Read{record: None}`.
  - A watch that does not decode gets `Event::WatchRefused{why: WatchRefusal::Unknown}`.
  - Answers go out through `clients::application(Delivery)` (clients/translation.rs:91-116) as `FromApplication::{Delivery, Read, Answered}`, routed at route.rs:61-69.
- **Application credentials ("the application's table"):**
  - Engine: `Values { accounts: Table, application: Map<u32, Secret> }` (values.rs:7-11); only grants at generation 1 resolve (values.rs:18-21). It is filled with exactly one credential, number 1, from the `application.credential` file (startup.rs:373-378).
  - Announcement: `root::Event::Credential{number:1, generation:1, valid: MAX}` is queued in `events` at `Engine::new` (engine.rs:59-61).
  - Worker: `link::Request::Value{table: Application, …}` goes into `Worker.application: Table` (worker.rs:281-288). The worker's routing never reads that table, so temper's identities' table goes there.
- **Payload version:** hard-coded `1` in two places. `kit::Component::new(limits, workers, 1)` (workers.rs:48; signature `new(limits, workers: Box<[Worker]>, payload: u32)` at jig-protocol-engine-workers component.rs:38). And `link::Credential { name, secret, payload: 1 }` (worker.rs:95; struct at jig-protocol-worker-link component.rs:20-24).
- **Periods from the smallest source:**
  - startup.rs:405-408 pushes `(project.number, project.policy.period_spend)` into `first_periods`.
  - engine.rs:344-348 steps `core::Event::Period{project, period: 1, budget, next: None}` once `domain.ready()` and not blocked. This comes after `RestartBegin` and before queued events and parties.
  - store.rs:468 truncates `first_periods` when the core Deployment row is restored, so later starts give none.
- **Host tools, charter and results (what temper-engine-smith replaces):** all of `tools.rs` (420 lines) uses `jig_protocol_tools` (`kit::decode/render/problem/declarations`) and `jig_protocol_charter` (`call_key`, `result`, `charter`, `encode`, `run_identity`). Call sites to rewire:
  - route.rs: :28 (`Request::ReadResult` → `tools::result`), :72 (forget on Relayed), :176 (`Now::Call` → `tools.decode`), :178 (`Now::EffectAnswer` → render), :183 and :185 (forget), :255 (`Held::CallAnswer` → render, durable).
  - workers.rs: :25 and :65-75 (`EndpointName` list), :146 (`tools::charter` in `encoded_assignment`), :312-319 (worst case).
  - engine.rs:16 and :69; limits.rs:101 (`tools::worst_case`); accounts.rs:350 (`tools::number`).
- **Replaced wholesale for temper:** `system.rs` (the test connector's system, answered in memory, route.rs:36-46), `family/*`, and the jig-test-domain, jig-test-worker and jig-test-store types.
- **Worker routing:** `worker_route.rs` (smith decode, encode, facts, process events) is generic smith routing and can be copied as is.
- **Profile:** `limits::for_profile(Profile, &jig Limits) -> Limits` with application constants `SMALL` and `STANDARD` (limits.rs:8-73). The application's profile follows `engine.limits.profile` (startup.rs:136-146). The reference has no separate application limits member.

## 2. `jig-protocol-config` (5,069 lines including tests)

**Parse API**
- `document(input, &DocumentLimits{bytes, tokens, depth}) -> Result<Document, Problem>` (document.rs:12).
- `engine(&Document, at: u32) -> Result<Engine, Problems>` (lib.rs:57) and `worker(&Document, at) -> Result<Worker, Problems>` (worker.rs:43). Both parse a member at a token index, and their problem paths are relative to that member. The service re-prefixes them with `engine_path` (startup.rs:30-36).
- Also: `files(&Engine) / worker_files(&Worker) -> Box<[File{at, path, max, private}]>`, `core(...)`, `secret`, `digest`, `handed_in`, `fits(memory, &[Share]) -> Result<u64, What>` (returns `What::Memory{over}`), `stop_bound`.

**Readers an application can reuse:** yes, in `pub mod read` (read.rs).
- `Object::new(&Document, at) -> Result<Object, What>`, `.member(..) -> Option<u32>`, `.required(doc, name, &Path, &mut Problems)` (pushes Missing), `.finish(doc, &Path, &mut Problems)` (pushes Unknown and Duplicate).
- `text(doc, at, max)`, `integer(doc, at, max)`, `boolean`, `duration` (milliseconds), `list(doc, at, max)`, `after`.
- Readers return `What`; the caller builds `Path::root().member(b"..").child(Segment::Index(i))` and pushes into `Problems` (capped at `PROBLEMS = 32`, overflow counted in `past`).
- There is no `Display` for `Path` or `Problems`.
- There is no generic overrides parser for an application's own limits: `overrides::apply(&mut jig Limits, name, value)` only knows jig's.

**Document shape:** jig's parser does not define the top level; the application does. The reference reads:
- `engine` (required), `application` (required), `workers` (optional);
- `service.components` ⊂ {accounts, workers, clients} (startup.rs:64-121);
- `application.{credential, connectors[2]}` (:147-163) and `application.workers[i].root` (:456-510).
So temper does not need to split the document; its startup parses `application` itself.

**Profiles:** `enum Profile { Small, Standard }` and `profile(Profile) -> Limits` (profile.rs:8-50). It is selected by `engine.limits = {"profile", "overrides"}`; any other name is refused (overrides.rs:485-502).

**Limits checks at start:** `limits::worst_case` plus trust roots plus configured endpoints are checked with `config::fits` against `engine.memory_bytes` (startup.rs:322-343), and each worker's against its own `memory_bytes` (:585-614).

## 3. `testing/jig-test` (982 lines)

**Files:** main.rs 12, src/lib.rs 287, host.rs 66, src/tests.rs 46 (3 usage and file tests), tests/support/mod.rs 338, tests/process.rs 22, tests/live.rs 140, Cargo.toml 60 (own lints; dev-deps serde_json and tempfile), clippy.toml 11.

**Commands:** only `serve CONFIG.json` and `agent AGENT.json` (lib.rs:17 USAGE, :20-23 Command). `agent` is `smith_agent_shell::run(&path, Box::new(stderr))` (lib.rs:43; smith crates/smith-agent-shell/src/agent_shell.rs:420). There is no `open` or `sign-in`.

**`serve` in order**
1. `skein_shell::read_file(path, CONFIG_BYTES = 1<<20)` (lib.rs:16,59-61).
2. `prepare` (:184-253):
   - Parses `engine` and `workers`.
   - Reads every name from `named()` with `read_private_file(path, max, effective_user)` (:62-72). Every named file must be private, charter instructions and the agent configuration included.
   - Builds the agent from smith's configuration via `smith_local_service::profile::host_limits` (:73-116; smith crates/smith-local-service/src/profile.rs:15).
   - Opens the worker roots with `open_root` (no creation).
   - Builds `AccountsRuntime` only if `service.components` lists "accounts", with `open_root(".")` as its root (paths are resolved against the working directory).
   - Seeds: `seed()`, `crypto_seed()`, and a deployment id from `crypto_seed()[..16]`.
   - Calls `startup`, then `make_root(engine.deployment.directory, 0o700)` and `adopt_store`.
3. `serve` (:254-284): `open_termination_signals` → `adopt_signals` → `Kernel::open(Config{operations})` → `skein_shell::drive(&mut kernel, &Clock::new(), &mut Host::new(service))`.

**`Host`** (host.rs): `iterate` calls `jig_test_service::iterate`. `drain()` writes each `service.links()` entry plus a newline to stdout and flushes; a write failure triggers a shutdown (:52-65). `exit()` returns `Code(u8::from(output_failed || end != Drained))` (:46-51). Errors print as `jig-test: {why:?}`, one Debug line (main.rs:8).

**Fixture configuration** (support/mod.rs:34-65): built from `jig_channel_world::fixtures::{engine_document(), WORKER_DOCUMENT, files(false)}`.
- It patches: `workers[0].engine = {"in_process":"workers"}`, agent program set to the binary with arguments `["agent", agent.json]`, `service.components = all three`, `hosts.listener = in_process`, `clients.listener = 127.0.0.1:0`, and the LLM endpoint at 127.0.0.1:34443.
- The agent's JSON is at :24-33 (TLS, `trust_der = root.der`).
- Handed-in account: `accounts/chat.json`.

**Process test** (one test, `a_killed_jig_test_restarts_and_the_work_is_found`):
- The binary runs under `skein_world::end_to_end::Binary` with pipes.
- The fake LLM peer (`smith_agent_process_world::fake::configured_transport(…, Transport::Tls)`) and the person (`jig_client_protocol_world::native::NativeClient`, which reads the link printed on stdout) are hosts in the test's `real::World`.
- The referee sends SIGKILL once the person's script ends.
- Run 1 is `answered_chat` → (1,1); run 2, with no peer, is `found_chat` → (1,0).
- Deadline 60 s (280 s under the `live` profile).

**Exclusion from the suites:** `.config/nextest.toml` default and fuzzy filters both exclude `binary(/^(process|live)$/)`.
- `[profile.process]`: `binary(=process)`, global-timeout 2m, test-threads 1.
- `[profile.live]`: `binary(=live)`, global-timeout 5m, test-threads 1, retries 0.
- `scripts/check.sh` runs fmt, clippy `--all-targets` (which compiles both tests), and nextest default plus fuzzy.

**Scripts**
- `scripts/proof-one-process.sh`: `cargo build -p jig-test --bin jig-test`, then `cargo nextest run -p jig-test --profile process`.
- `scripts/proof-live.sh`:
  - Uses `umask 077`; the log directory is `${JIG_RUN_LOGS:-./logs/jig-26}`.
  - Python reads `~/.codex/auth.json` read-only and refuses if the token's `exp` is within 300 s.
  - Writes `{access_token, expires_at, account_id}` to a `mktemp` file, chmod 600, removed by an EXIT trap. No refresh token is copied.
  - Picks the model from `JIG_LIVE_MODEL`, or from `smith/benchmarks/agents/models.toml [small.codex]` found by walking up from the working directory.
  - Resolves chatgpt.com, writes `live-config.json`, then runs `timeout 300s cargo nextest run -p jig-test --profile live`.

**Live test:**
- Patches the handed-in account file, the codex endpoint and TLS, the charter (turns 5, time 240 s), and the agent configuration (smith trace "capture everything").
- Counts completions from the transcript frames at `state/transcripts/0…0/0…1` and asserts 1..=5.
- Asserts the captured prompt holds the charter and the words.
- The budget is enforced only by charter turns, the script timeout and this assertion.

## 4. `tests/one-process` (1,108 lines)
- **Files:** src world 337, referee 381 + referee/agent 72, fixtures 92, world_signals 75, lib 6; tests one_process.rs 47, fuzzy_one_process.rs 17.
- **Composition** (world.rs):
  - The service comes from `jig_test_service::startup(fixtures::document(), files(), Some(channel::agent()), channel::seeds(), None, Some(WorkerRuntime))` (:135-164). The document is made by string-patching the channel fixtures (fixtures.rs:86-104).
  - Store: on `skein_fake_machine` (Kill closes handles, PowerLoss is `crash(seed)`, :170-197), observed by `jig_channel_world::world::store::Store`.
  - Agents: smith agents as a hosted program `agent-program` (`smith_agent_process_world::configured`).
  - LLM: `skein_fake_peers::llm::Peer`.
  - People: `NativeClient` with `Script`. The referee pops `service.links()` and calls `sign_in` (referee.rs:259-265).
  - Referee: jig's conformance referee.
- **Stories:** chat answered; drain mid-run; next start settles claims; kill before a commit; power loss mid-run. The fuzzy test runs 7 seeds × {Kill, PowerLoss} with a drawn cut point.
- **Cost:** the pinned plan aims at 0.6 s focused and 3 s fuzzy in all, each focused test under 0.5 s (docs/development/plans/jig-protocol/jig/26-one-process.md:141). There are no measurements at the pin. jig main 3eb97646 (newer than the pin) records the power-loss test at 0.81 s, then 0.67 s alone after a fix (docs/development/plans/test-budgets/findings/cost-jig-units.md:7).

## 5. `tests/channel` (3,430 lines)
- It runs jig's own reference application (jig-test-service and its test domain, store and connector; fixtures.rs carries the ENGINE document, `WORKER_DOCUMENT` and `engine_document()` with `application.{credential, connectors}` and `service`).
- It needs nothing from an application. It has 26 focused tests in tests/channel.rs.
- Since the reference sends and drops empty parts, this world carries no parts at all. Non-empty opaque parts are exercised only in tests/worker-link (fixtures.rs:214-215,434) and tests/engine-workers (fixtures.rs:367).

## Discrepancies
1. **No "system module" holds the one-process composition.** Design: "temper-service's system module owns the engine's protocol layer and the worker's" (protocol.md §8; §4 tree "one loop: temper-service's system module"; §5 "an engine, a worker and a system module, as jig's reference service is"). Code: `system.rs` is "The test application's fault-free system, answered in memory" (system.rs:1); the composition is lib.rs (`Service`, `iterate`, `route_events`).
2. **Only two of the four commands have a template.** Design: "the four commands of section 4, copied from jig's reference binary" (§5). Code: jig-test has only `serve` and `agent` (lib.rs:17-23). `open` and `sign-in` have no template; skein's `component/sign_in.rs` is a component, not a command.
3. **Problems are not printed one per line with paths.** Design: "Every problem found is reported, one line each on stderr", with paths like `application.forges[0].user` (configuration.md §2). Code: there is no `Display` for `Path` or `Problems`; jig-test prints `startup: {why:?}` on one Debug line. temper must write the formatter.
4. **jig's document has a fourth member that gates components.** temper's document is `{engine, workers, application}`; jig's reference also reads `service.components`. Without it, accounts and clients are off (startup.rs:72-101,120), and jig-test builds `AccountsRuntime` only when "accounts" is listed (lib.rs:~227-236). The copy must enable all three unconditionally.
5. **The identity credential is not announced before the restart.** Design: "for each identity as its first events, before its restart" (credentials.md §4; configuration.md §8). Code: the credential is queued at `Engine::new` (engine.rs:59-61), but `RestartBegin` is stepped first (engine.rs:338-340).
6. **The worker root is not created.** Design: `root` "made private (0o700) if absent" (configuration.md §4). Code: jig-test opens worker roots with `open_root`; only the deployment directory uses `make_root(…, 0o700)`.
7. **The fake LLM peer is not "a process of its own".** Design (testing.md §2.6, echoing jig's service.md §8): "a process of its own on loopback". Code: it is a host in the test's single-threaded `real::World`, on the fixed port 127.0.0.1:34443, not an OS process.
8. **tests/channel carries no parts.** Design: "jig's channel worlds carry them as opaque bytes" (channel.md §9). Code: tests/channel carries none (parts are empty throughout jig-test-service, see section 1); only tests/worker-link and tests/engine-workers carry opaque parts.
9. **Budgets and constants need changing in the copy:**
   - Live: jig's script refuses tokens expiring within 300 s and runs `timeout 300s`, the live profile has a 5m global-timeout, and the test asserts at most 5 completions. temper's 15 min and 25 completions need all four changed.
   - Process: jig's profile `global-timeout = "2m"` against temper's "aims at 3 minutes".
   - The payload version is hard-coded `1` in two places (workers.rs:48, worker.rs:95).
   - Only one application credential is supported (startup.rs:378).
   - Both jig process tests use `tempfile::tempdir()`, the system temp directory, and the live test keeps it (`directory.keep()`) along with prompt traces.

Two more things the plan should account for:
- **Token prefixes for temper's components:** the forge and git components need prefixes of their own. Free top nibbles are 5-7 and 11-15; nibble 3 is already shared by accounts and workers.
- **File operations for the git component's `Make`:** the worker routes only its root-close kernel op (worker.rs:571-592), so `Make` needs new file-op routing.
- **The person harness has to be rebuilt:** jig's is `NativeClient`, 665 lines in tests/client-protocol/src/native*, bound to `jig_test_domain_client`. temper needs its own equivalent over temper's client root.
