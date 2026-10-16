# Webhooks: how far temper's main is from hints (temper 0ff45022e, 2026-10-16)

Paths are under ~/src/rust/temper. The graph does not track protocol-forge, fake-forge-protocol or throwaway-forgejo: all facts are from source.

## 1. Configuration
- `application.webhooks` is refused, `null` included: crates/temper-service/src/startup/application.rs:342-344 pushes `What::NotBuilt` ("reserved: not
  built yet", startup/problems.rs:14,74) when the member exists; pinned by src/tests/startup.rs:43 and :106-109. The design's own example
  (configuration.md:540 `"webhook_secret": null`, :570 `"webhooks": null`) therefore does not load.
- `webhook_secret` is not decoded: `fn forge` (application.rs:175-231) reads number, kind, api, transport, git, api_token, user, identity, then
  `read::finish` (:195) rejects any other member as unknown. `Forge` (:21), `files()`/`resolve()` (:416-538: api_token, tls trust, identity token
  only) and `settings::Forge` (settings.rs:7-20) carry no secret, no private-file read, no bound. `user` is a plain u64 (no `{"file": path}`).
- Not checked: `webhook_secret` with `webhooks` null; the listener on loopback and unique. The clients' listener has a loopback check
  (startup/checks.rs:39-42) and its sockets counted at :272 (connections + 3); nothing for hooks.
- Limits: `kit::Limits` (crates/temper-engine-protocol-forge/src/limits.rs:9-48) has no `hooks`, `hook_head_bytes`, `hook_body_bytes`,
  `hook_deadline`, `hook_secret_bytes`; `Unusable` (:85-116) has no hook variants; `small()` (temper-service/src/forge.rs:110) sets none;
  limits/names.rs:169-211 has no `forge_component.hook*` name (an unknown override refuses the start, :212). configuration.md:346-350 names them.
- Keep: `poll`, `poll_max`, `hinted`, `slow` exist (client limits.rs:37-40, checked :65-68; 5 s, 30 s, 1 s, 40 s at temper-service/src/limits.rs:74-81;
  override names `forge.client.{poll,poll_max,hinted,slow}_ms`, names.rs:105-108), so the hinted story can raise them today. `settle` exists
  nowhere (no field, no name); configuration.md:345 names `forge.client.settle_ms`. Section 7 lists none of the four by name (only the prefix, :331).

## 2. Domain
- Types match the design: domain-forge-client/src/api.rs:20-37 `Hint { repository, change, key: Option<Box<[u8]>> }`; `Change::{Item(u64),
  Branch(Box<[u8]>), BranchMoved { branch, head: Commit, actor: u64 }, Commit(Commit)}`; `Commit = [u8; 32]` (:12).
- Entry: root `Event::Forge` (engine-domain/src/route.rs:77-83, drops a hint before `domain.ready()`, as forge.md 5 says) -> forge domain
  `Event::Hint` (domain-forge/src/boundary.rs:593; domain.rs:791-798: wakes the repository's changes, `ci_hint` :2896 reads CI once for a `Commit`
  hint when a `Topic::Ci` subscriber exists, then forwards) -> client `Event::Hint` (client/src/boundary.rs:243) -> `keep::hint` (keep.rs:622).
- Who sends one: no production code. The component's `Event` is `Answered | Stopped | Closed` (protocol-forge/src/boundary.rs:37-50);
  `forge_pass` (temper-service/src/engine.rs:385-408) handles those three; `forge::event`/`back` (service/forge.rs:93,101) wrap and recover answers only.
  Only tests send hints: tests/conformance/src/systems.rs:224-245 (fake `Request::Hook` -> `Change`, key None), tests/engine/forge/client/src/world.rs:585,592,
  tests/engine/domain/tests/forge_routes.rs:1618+, client tests/keep.rs:248+. No test fills `key`.
- A hint reads within `hinted`: `nudge_resource` (keep.rs:701-711) arms the resource alarm at now+`hinted` and resets its interval to `poll`; the repository
  listing is armed at now+`hinted` (:660-671) or flagged when busy. `affected` (:674): Item by number (a Repository row always), Branch/BranchMoved by
  branch or a pull's head/base, Commit by a pull's commit/base_commit or a branch tip. A repository or resource not held is ignored.
- Echoes: only the pusher. `BranchMoved` keeps `pushed_by` (keep.rs:650-652), read by `branch_observed` (:1147) for drift. A key is length-checked and
  otherwise ignored (keep.rs:638-641: a copied marker must still cause a read); forge.md 5 (:617) says the domain drops echoes by key. Differs in kind.
- `settle` is not built (domain/forge.md 5.2, 8.2 say so): no re-read at `hinted` doubling of a not-mergeable pull or a hinted job's pending CI. Worse:
  forge/src/domain.rs:2025-2029 maps `mergeable == false` to `Status::Failed`, which change/src/domain.rs:482-492 sends to `resolving_due`: a pull read
  not mergeable just after a push is taken as conflicted today. The `Pending`/`Unknown` arms exist, but nothing makes them for a pull that was read.

## 3. The forge component, and how temper-service hosts listeners
- crates/temper-engine-protocol-forge (no_std) uses `skein_http::client` only (connection.rs:77): no server, no listener. `Component::up`
  (component.rs:178) routes io events by `self.connections` (forge connections) and ignores `Listening`/`Accepted` (:238-247); `MAX_OUT` 16/16 (:22).
  `Settings` has `forges` only (settings.rs:7-10). A listener is new code beside connection.rs: a webhook-connection slab (HTTP server, JSON decoder,
  HMAC), settings (`webhooks` address, per-forge `Option<Secret>`), limits and `Unusable` variants, `worst_case` (limits.rs:311), `Event::Hint`, and a
  way back for refused or returned hints (`Request::Refused/Returned` hold `Answered` only).
- skein (pinned 8a10437) has `skein_http::server` (skein-http/src/server.rs). skein-lib has no SHA-256, `sha256::Hmac` or `equal`; skein-io has an
  incremental SHA-256 `digest` (skein-io/src/digest.rs:90) and no HMAC; skein main (ec686b8) only designs them (lib.md 6.2; skein STATUS.md:34,93-95: temper's
  forge-setup plan takes them). temper's only SHA-256 is external `sha2` 0.10.9 (Cargo.toml:125; domain-forge-issues/src/domain.rs:8). See findings/skein-sha256.md.
  temper-forge-forgejo has no delivery or org-hook documents (no match for hook, delivery or signature in its src).
- The clients' listener: jig's clients component issues `io::Request::Listen` itself (temper-service/src/clients.rs:51-100); temper-service tags its io
  owners with a token prefix (`tag` :266, `owns` :262; tokens.rs CLIENTS 4<<60, FORGE 5<<60), pushes `engine.clients.io` into io (system.rs:289-294) and routes
  events back by `clients::owns` (system.rs:504-509).
- The forge's io goes through `forge::tag` (service/forge.rs:249), which tags `Connect` only; prefix-5 events reach `Layer::up` (forge.rs:165; system.rs:501-503),
  untagged by `accounts::untag` (accounts.rs:470). A second listener: extend `tag` to `Listen` and `Bind`; pass `webhooks.listener` (loopback, nothing to
  resolve) into component settings (the binary builds `ForgeRuntime`, crates/temper/src/lib.rs:264-298); forward hints in `forge_pass` (engine.rs:385) as
  `root::Event::Forge(Event::Hint)`; add the sockets to checks.rs:272 and the memory sum (system.rs:575-600; kernel operations not verified).

## 4. The fake forge's protocol layer (testing/temper-fake-forge-protocol)
- It cannot send deliveries: server only (`Server`, server.rs); `Service::target` answers `Err(request)` for `Request::Hook` (service.rs:94) and
  `server::down` returns on it (server.rs:140). No client, no hook URL or secret in `Config` (lib.rs:30-47) or `Repository` (lib.rs:23-27), no HMAC.
- The fake domain does make hooks: `Request::Hook { repository, change, number, branch, commit: Option<u64>, by }` (fake-forge-domain boundary.rs:41),
  `Change::{Issue, Pull, Comment, Review, Status, Push, Wiki}` (api.rs:720), only for a repository `Setup { hooked: true }` (api.rs:752; hooks.rs:63),
  after a drawn latency, late or lost (hooks.rs:66-91; `Config` hook_min/max, hooks_late, hooks_lost). `hooked: true` is set only in
  tests/conformance/src/systems.rs:76 and tests/fake-forge/tests/memory.rs:118; the forge worlds set false and discard `Hook`: engine/forge/src/world.rs:358,530,
  engine/forge/client/src/world.rs:467,645, protocol-forge/src/joint.rs:197, one-process/src/forge.rs:188 (dropped, since `target` is Err).
- Gaps against forge.md 5 and 9: a hook has no body key, event name or job payload (`run`, `commit_sha`, `trigger_user`); `Status` goes out for every fake
  CI status (ci.rs:238, pending too) where the design sends a job's end; `Wiki` is retired; the fake's `mergeable` (pulls.rs:347-352) is computed at once, so
  no "checking" phase exists to exercise `settle`.
- Driven by worlds: `fake::Domain` with `Setup`s, a `protocol::Config` (tokens, users, repositories, faults), `Service::new`, one `Server` per connection
  fed bytes (`Pipe`, protocol-forge/src/joint.rs; simulated sockets, one-process/src/forge.rs).

## 5. The throwaway Forgejo (testing/temper-throwaway-forgejo)
- `app.ini` is one format string, `ini()` (instance.rs:261), pinned byte for byte by src/tests.rs:6-11: server on loopback, sqlite, repository, security,
  service, log, git, `[actions] ENABLED`, optional `[api] MAX_RESPONSE_ITEMS`. No `[webhook]`, and `Settings` (lib.rs:9-15: max_response_items, actions, git,
  ready_polls, answer_bytes) has no field for one: `ALLOWED_HOST_LIST` cannot be set, and Forgejo's default refuses loopback. A new field, a line in `ini()`
  and the exact-string test would do it.
- No hook support: `Instance` offers port, base, scratch, admin, create_user, create_repository, add_collaborator, exchange, secrets, stop (instance.rs:143-210);
  the generic `exchange` could POST a hook, nothing does. No runner support. tools/forgejo-conformance records `webhook-deliveries` as
  `Unchecked("webhooks are built after standalone")` (src/lib.rs:29-30, facts.txt:76): no recorded headers, signature, payload fields, job-status ordering or
  mergeability timing (forge.md 13, setup.md 13.3).

## 6. Worlds and tiers a webhook story could run in
- Forge connector world, tests/engine/forge (src/world.rs, client/src/world.rs): fake domain and connector, no component, no bytes; `Hook` discarded.
- Machine world, tests/engine/protocol-forge (src/machine.rs `Machine`, tests/machine.rs, fuzzy_machine.rs): real `Component`, scripted peers, io played by
  the world; a `Listen` is a panic there (machine.rs:225), so it needs an accept path. The natural home for scripted deliveries (forge.md 10).
- Protocol world, tests/engine/protocol-forge (src/joint.rs `Joint::with` :56, src/protocol.rs `Protocol`, tests/protocol.rs, fake_protocol.rs, fuzzy_protocol.rs):
  `Component` + fake `Service/Server` over `Pipe`; a `Listen` panics (joint.rs:261); the fake would have to become a client to deliver.
- One-process world, tests/one-process: real `temper_service` and fake processes on a simulated kernel; `ForgeHost` (src/forge.rs) is the fake forge server on
  127.0.0.1:3000 (spawned world.rs:611,682,689); `fixtures::forges()` (fixtures.rs:198) is the `ForgeRuntime`; the document (fixtures.rs:80-105) has no `webhooks`;
  `hooked: false` (stories.rs:157). Needs the listener on the service side and a forge host that connects out (it only listens).
- Process tier: scripts/proof-one-process.sh builds `temper` and runs `cargo nextest run -p temper --profile process` (.config/nextest.toml `[profile.process]`,
  binary `process`, 10 min, 1 thread); scripts/check.sh excludes it. crates/temper/tests/process.rs has 6 tests; the landing story (:71) uses
  `support::forgejo::Throwaway::start` (support/forgejo.rs:13, `actions: false`) and `support::configuration` (support/mod.rs:19), no webhooks, landing rule
  `"ci": false` (forgejo.rs:132), the component started by `temper serve`, polling only. The binary has only open, sign-in, serve, agent (crates/temper/src/lib.rs:45-57):
  no `temper forge prepare|run|reconcile`, so a hook there comes from the test (`exchange`). Not verified: running the tier (needs TEMPER_FORGEJO).
