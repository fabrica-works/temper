# Audit: jig-components

2026-10-14, read only, by a sub-agent of the planner, at temper `cc2697cf` with jig `d91a89c4`, smith `dfc0fe57` and skein `b5d3a2c3` (temper's pins; jig's, smith's and skein's paths are relative to their pinned checkouts). Evidence the plan cites (development.md, 4.1 and 5.4): where it and landed code differ, the landed code wins. The design gaps it found are closed by the Markdown landing "Design: close the gaps the protocol plan's code audit found"; `README.md` here maps each finding to its increment.

I audited all five jig areas at d91a89c (skein at b5d3a2c). Most of what temper's design needs is there, but the design gets several things wrong about who does what (listed under Discrepancies). Paths below are relative to the jig pin, `S:` marks the skein pin, and `temper:` marks the temper repo. I edited no files.

## 1. jig-protocol-store
- **Tag:** `keys::APPLICATION: u8 = 2` (`crates/jig-protocol-store/src/keys.rs:13`). `pub fn application(key:&[u8], limit:u32)->Option<Box<[u8]>>` (:76) writes `[2] ++ key` raw, with no escaping. `of_application(bytes)->Option<&[u8]>` (:85) strips the tag. `pub mod keys` is at `src/lib.rs:212`.
- **Writes**, in `src/boundary.rs:36-49` `enum Write`:
  - `Save(core::Record)`, `Erase(core::Key)`, `DropTranscript{task}`
  - `Put{key:Box<[u8]>, value:Box<[u8]>}` (key without the tag)
  - `Remove{key:Box<[u8]>}`
  - They go inside `Request::Commit{number:u64, writes:Box<[Write]>}` (:19), mapped to kv ops at `src/commits.rs:102-103`.
- **Restore:** `Request::Restore{owner:Token, range:Restore, after:Option<Cursor>, rows:u32, bytes:u64}` (`boundary.rs:21`).
  - `enum Restore{Jig(Box<[RestoreRange]>), Application(Box<[skein_kv::Range]>)}` (:53-60); application ranges are relative to the tag.
  - `skein_kv::Range{start:Box<[u8]>, end:Option<Box<[u8]>>}` and `Range::prefix(&[u8])` are at `S:crates/skein-kv/src/boundary.rs:34-52`. "Every key under temper's tag" is `Range{start:Box::new([]), end:None}`, which `pages::application` turns into `[2]..[3]` (`src/pages.rs:112-123`).
  - Answers: `Event::Restored{owner, rows:Box<[Row]>, next:Option<Cursor>}` (`boundary.rs:184`), with `Row::Application{key, value}` (:134, key relative).
  - `Cursor{interval:u32, after:Box<[u8]>}` (:123) is exclusive. Its `after` is the **tagged** kv key (`pages.rs:52`, `after: row.key.clone()` comes before the tag strip at :128).
  - `Event::Corrupt{owner}` stops the engine.
  - The reference routing (`testing/jig-test-store/src/route.rs:119`) uses `RestartStep::RestoreConnector{connector}` → `Restore::Application(Box::new([live(connector)]))`, continues by passing the store's cursor back (:283), and checks that each decoded record's key re-encodes to the row's key (:273).
- **Version and sizes:** `sizes::Application{version:u32, key:u32, value:u32}` ("before the component's key tag") is at `src/sizes.rs:7-11`. It is passed to `Store::over(limits:&Limits, application:sizes::Application, stores:Stores)->Store` (`lib.rs:68`) or `Store::on_directory(&Limits, sizes::Application)` (:93).
  - The format row is `format::current(application:u32, deployment)` (`src/format.rs:6`). A mismatch gives `Event::OpenRefused{why:OpenRefusal::Format{family:Family::Application, found, wanted}}` (`boundary.rs:139-152`, `src/open.rs:47-50`).
  - The size check is a **free function the caller runs**: `sizes::check(domain:&core::Limits, application:&Application, journal:&JournalLimits, store:&skein_kv::Limits, orphan_erases:u32)->Result<Sizes,SizeRefusal>` (`sizes.rs:33`).
  - `Sizes{key,value,ops,commit}` are then copied into the kv limits (reference: `testing/jig-test-service/src/store.rs:39-68`). The key size adds 1 for the tag (`sizes.rs:79`).
  - The reference application: `jig_test_store::STORE_VERSION: u32 = 1` and `application(&connector::Limits)->sizes::Application` (`testing/jig-test-store/src/lib.rs:14,26`).
- **Memory mode:** `Stores{store:skein_kv::Store, secrets:skein_kv::Store}` (`lib.rs:41`), built with `skein_kv::Store::memory(&Limits)` (`S:crates/skein-kv/src/store.rs:164`). `on_directory` uses `Store::new` and the disk path (`lib.rs:93-99`).
  - In memory mode, `Request::Open{owner, root:Root}` still takes a `Root{token, fd}` for transcripts (`src/directory.rs:55-58`).
  - Transcript file requests still leave on `Below.transcripts` (`boundary.rs:246`), so a world must answer them.
- **World swap (protocol/store.md §14):** `testing/jig-test-store-files` ("The real store over memory and skein's crashing file machine").
  - `Store::new(&jig_test_domain::Limits)` (`src/store.rs:62`), `submit(number, writes, delay)` (:107), `tick(hold)->Result<Option<u64>,u64>` (:117), `crash(seed)` (:148, rebuilds from `into_stores()`), `fork()` (:171), `ask(StoreOut)` (:198), `answers()->Vec<Up>` (:211), `component_limits(&root::Limits)->Limits` (:348).
  - It implements `jig_conformance::Durable` (`src/durable.rs`; trait at `testing/jig-conformance/src/harness.rs:10-25`).
  - `Files` (`src/files.rs:9-46`: `new`, `root()->Root`, `crash(seed)`, `answer(file::Request)->Arrived`) answers file requests on `skein_fake_machine`.
  - It is hard-wired to `jig_test_domain` / `jig_test_store::route`. jig's core world re-exports it (`tests/core/src/lib.rs:16`).

## 2. Worker wire, engine-workers, worker-link
- **Payload version (u32):**
  - Engine side: `engine_workers::Component::new(limits:&Limits, workers:Box<[Worker]>, payload:u32)->Result<Self,Unusable{Limits,Workers}>` (`crates/jig-protocol-engine-workers/src/component.rs:38`).
  - Worker side: `worker_link::Credential{name:Box<[u8]>, secret:Secret, payload:u32}` (`crates/jig-protocol-worker-link/src/component.rs:20`) → `worker_link::Component::new(limits:&Limits, credential:Credential, seed:u64)->Result<Self,Unusable{Limits,Credential}>` (:54).
  - Wire record: `Credential{name:text 256, secret:secret 1024, payload:u32}` (`crates/jig-protocol-worker-wire/schema/worker-wire-v2.schema:4`).
  - Checked in `opening.rs:25`, after name and secret (else Unauthorized) and before Busy: `Reason::Application` = 257 (worker-wire `src/lib.rs:51,65`). The worker sees `Request::Fact{why:Closed::RefusedPeer(257)}` and redials.
  - The reference passes the literal 1 on both sides (`testing/jig-test-service/src/workers.rs:48`, `worker.rs:95`).
- **Parts:** `wire::Part{token:Token, bytes:Box<[u8]>}` (worker-wire `lib.rs:40`). Host vocabulary (`crates/jig-domain-host/src/boundary.rs`):
  - `Workspace{workstream:u64, items:Token}` (:472)
  - `RunAssignment{identity:[u8;16], run, attempt, workspace:Option<Workspace>, save:bool, charter, grants:Box<[Grant]>}` (:391)
  - `Work{left:Option<Token>, saved:Option<Token>}` (:571)
  - `Preparation::{Transient, Permanent{resource:Option<Token>}}` (:617)
  - `SettledAnswer::Delivery{outcome, evidence:Box<[u8]>}` (:428): evidence is inline bytes, not a token.
- **Engine-workers events and requests** (`boundary.rs`):
  - `Event::Down{channel:Token, down:host::LinkDown, parts:Box<[Part]>, values:Box<[Option<GrantValue>]>}` (:30). The routing supplies the items part by token, plus one value per grant in grant order.
  - `Request::Up{channel, up:host::LinkUp, parts:Box<[Part]>}` (:59). The component mints tokens for left, saved and resource (`translate.rs:189-264`).
  - `GrantValue{value:Secret, account_id:Secret}` (:14).
  - A `None` value **drops that grant** from the assignment (`translate.rs:455-456`). `LinkDown::Grant` with `None` drops the frame, counted by `dropped_down()` (:406, `component.rs:152-155`).
- **Worker-link** (`boundary.rs`):
  - In: `Event::Up{up:host::LinkUp, parts:Box<[Part]>}` (:10).
  - Out: `Request::Down{down:host::LinkDown, parts:Box<[Part]>}` (:35), carrying only the items part, with a token the link mints (`translate.rs:440-441`).
  - Values come out as `Request::Value{table:host::GrantTable, account:u32, generation:u64, expires:Time, value:Secret, account_id:Secret}` (:39, `translate.rs:397-412`). The link keeps no table.
  - The reference worker keeps them in `jig_protocol_accounts::table::Table::new(credentials+1, token_bytes)` (`testing/jig-test-service/src/worker.rs:125`, :281-288).
- **Grants:**
  - Host: `host::Grant{table:GrantTable{Accounts,Application}, account:u32, generation:u64, valid:Duration}` (host `boundary.rs:384,669`).
  - Wire: `Grant{table, account:u32, generation:u64, valid:duration, value:secret 8192, account_id:secret 256}` (schema:98).
  - The reference engine's application table is the routing's own `Map<u32,Secret>`, generation 1 only, with `account_id: Secret::new(Box::default())` (`testing/jig-test-service/src/values.rs:10-32`).
- **Sizes:** `sizes::Sizes{charter, transcript, turn, message, sender, call_name, tool_name, call_input, answer, result, detail, grant, account_id, name, secret, model_name, items, left, saved, evidence, resource, slots, calls, turns, inbox, answered, models, workstreams, accounts, credentials, facts}`, all `u32` (worker-wire `src/sizes.rs:6-40`).
  - `sizes::limits(&Sizes)->Option<v2::Limits>` (:56) checks the ceilings (:95).
  - Ceilings (`generated/v2.rs:48-80`, schema:52-108): `items` 1 048 576; `left` / `saved` 65 536; `resource` 1 024; `evidence` 1 048 576; `grant` 8 192; `account_id` 256; `accounts + credentials` ≤ 64 (`assignment_grants`).
  - Both `Limits` structs carry `sizes: Sizes` (engine-workers `limits.rs:11-12`, worker-link `limits.rs:14`).

## 3. Accounts (no component in jig-protocol-accounts)
- **The crate** is a vocabulary over skein's driver:
  - `handed_in::hand_in(account:u32, token:Secret, lapses:Wall, account_id:Option<Box<[u8]>>, limits:&oauth::Limits)->Result<driver::Request,Unfit>` (`crates/jig-protocol-accounts/src/handed_in.rs:12`), which yields `Request::HandIn`.
  - `table::{Table::new(accounts,token_bytes):35, granted(account,generation,Value,expires)->Result<(),Unfit>:42, value(account,generation,now)->Option<&Value>:64, expire, next_expiry}`, `Value{token:Secret, account_id:Secret}` (:9), `ACCOUNT_ID_BYTES=256`. Slots are indexed by number, sized n+1.
  - `translate::{down(ToComponent)->driver::Request, up(driver::Event,now)->Up{Granted{event,account,generation,value,expires} | Child}}` (`translate.rs:9-58`).
- **The component** is `skein_oauth_accounts::Component::new(accounts:List<Account>, &Limits, CryptoRng)` (`S:crates/skein-oauth-accounts/src/component.rs:137`), with `Account::{HandedIn, SignIn{registration, endpoint, keeper}}` (`accounts.rs:15`).
- **Config:** `jig_protocol_config::llm::{Llm{endpoints, models, accounts, records}, Account{number:u32, source:Source}, Source::{SignIn(SignIn), HandedIn{file}}}` (`crates/jig-protocol-config/src/llm.rs:8-56`). The reference wiring is `testing/jig-test-service/src/accounts.rs:96-185`.
- **File format:** `forms::handed_in(bytes, &DocumentLimits)->Result<HandedIn{token:Secret, lapses:Wall, account_id:Option<Box<[u8]>>},Problems>` (`crates/jig-protocol-config/src/forms.rs:37-87`). It is a strict JSON object: `access_token` (text, required), `expires_at` (integer epoch seconds, required), `account_id` (text, optional), no other members.

## 4. Engine-clients and client wire (`crates/jig-protocol-engine-clients/src/boundary.rs`)
- **Wire records** (`crates/jig-protocol-client-wire/schema/client-v7.schema:3-6`): `Family{number:u16}`, `Owner{jig, family:Family}`, `Keyed{key:fixed 16, owner, version:u32, request:bytes 1 MiB}`, `Owned{owner, version, record}`. `VERSION:u32 = 7` (`src/lib.rs:12`). jig defines **no FORGE constant**; the reference uses family 1 (`testing/jig-test-service/src/clients.rs:98-119`).
- **Settings:** `Settings{listener:Addr, link:Option<people::Identity>, deployment_provider:u16, heartbeat, rows, served:List<Served>, providers:List<u16>, seed:Secret, stream:u64}` (:135) and `Served{family:Option<u16>, first:u32, last:u32}` (:148), where `None` means jig's own wire. Construction: `Component::new(Settings, &Env<Limits>, &mut Queue<Request>)->Result<_,Unusable{Limits,Seed,Listener}>` (`component.rs:47`).
- **An application request** goes out as `Request::Application(ToApplication::Request{reply_to:ReplyTo, sign_in:u64, key:[u8;16], family:u16, version:u32, record:Box<[u8]>, digest:[u8;32]})` (:69-78). The digest is over the record's bytes (`component.rs:698`).
  - Answer with `Event::Application(FromApplication::Answered{to, family, record})`, or `FromApplication::Undecodable{to}`, which gives HTTP 400 (`component.rs:124-130`).
  - An unserved family or version gets 400 from the component before the routing sees it (`component.rs:675`, `reads.rs:62`).
- **Reads and watches:** `ToApplication::Read{reply, sign_in, family, version, record}` is answered with `FromApplication::Read{reply, family, record:None}`, which means Unknown (`reads.rs:158`). `ToApplication::Watch{watcher, ...}` is refused with `Event::WatchRefused{watcher, why:core::WatchRefusal::Unknown}`. This is exactly what the reference does (`testing/jig-test-service/src/family.rs:127-177`).
- **Link:** `Request::Link{link:Box<[u8]>}` (:65). Its text is `http://` + dotted IPv4 or `[::1]` + `:PORT/jig/v1/sign-in/link?secret=` + unpadded base64url of 32 random bytes (`sign_in.rs:37-56,102-122`). It needs `settings.link=Some` and a loopback listener (else `Unusable::Listener`), and is minted at start or on `Listening`.

## 5. What an application root receives
- **jig has no generic type for it.** The routing gets `ToApplication::Request` (above). The root then hands its core `core::Event::Application{reply_to, sign_in, key:[u8;16], project:u32, connector:u16, digest:[u8;32], owner:Token}` ("admit by metadata; its owner retains the value", `crates/jig-domain-core/src/routing/vocabulary.rs:354`).
- **The rest of the flow:**
  - The core sends the connector `core::Ask::Application{request, token, owner, person, project, role, retention}` (:41).
  - The connector asks back with `core::Event::ApplicationPart{request, ask:Box<people::Ask>}` (:364) and gets `Ask::ApplicationPartAnswered{request, outcome}`.
  - It finishes with `Event::ApplicationFinished{request, outcome:Result<Token,people::Again>}` (:366).
- **Reference root:** its own `Event::Application{reply_to, sign_in, key, project, number, digest, value}` (`testing/jig-test-domain/src/lib.rs:182`) mints `owner` and pushes the core event (:621-637).
- **The adoption request** is `people::Ask::Adopt{project:u32, adoption:people::Adoption{resource:ResourceName{connector:u16, path}, role:ResourceRole, options}}` (`crates/jig-domain-people/src/boundary.rs:198-216,271`).

## Discrepancies
1. "makes jig's adoption request on the party's behalf (`adopt_repository_ask`)": no such function exists in jig. It is **temper's** `adopt_repository_ask(adoption:forge::Adoption, connector:u16)->Option<people::Ask>` (`temper:crates/temper-engine-domain/src/route/forge_route.rs:39`). It builds jig's `people::Ask::Adopt`, which travels as `core::Event::ApplicationPart`.
2. "hands the root the party's keyed request: …": jig has no type for this. temper's root needs its own variant (`PartyInput` has none today, `temper:crates/temper-engine-domain/src/boundary.rs:339-350` is only `Ask`). The core event it leads to also needs `project` and a root-minted `owner: Token`.
3. "hands them with `STORE_VERSION` to jig's component, which checks them … the start is refused": the component checks only the version, at open (format row). The sizes are checked by `sizes::check`, which temper's service must call itself and then copy the resulting `Sizes` into the kv limits.
4. "The routing encodes the root's cursor with temper's key encoder": jig's `Cursor.after` is the **tagged** key. The routing must build `Cursor{interval:0, after:keys::application(&k, limit)?}`, or pass jig's cursor back unchanged. `Row::Application.key` is relative (no tag).
5. "temper's engine worlds run jig's component over skein-kv's in-memory mode … in place of jig's fake store": jig ships no generic world driver. `jig-test-store-files::Store` is tied to the reference root, so temper must write its own. It can reuse `jig_test_store_files::Files` and `jig_conformance::Durable`. temper's store README §6 also omits that the component still issues transcript file requests and needs a `Root` in memory mode.
6. "`jig-protocol-engine-workers` fills it in … through the routing" / "fills the value in from jig's accounts' table": in code the **routing** looks values up and passes `values` in `Event::Down`. jig ships no engine-side application table (the reference uses a plain `Map`). A missing value silently drops that grant rather than refusing anything.
7. "jig's components carry each part as bytes under the token the hub's vocabulary names it by": true for items, left, saved and resource only. A delivery's evidence is inline bytes in `host::SettledAnswer::Delivery`, with no token.
8. "`jig-protocol-accounts` … holds the tokens lent": it supplies `table::Table`, but the routing owns the table instance. "Construction from config" is skein-oauth-accounts' `Component::new`, configured from jig-protocol-config's `Llm`. jig-protocol-accounts has no component.
9. "for the family `FORGE`, `CLIENT_VERSION` alone": correct as `Served{family:Some(FORGE), first:CLIENT_VERSION, last:CLIENT_VERSION}`, but FORGE and CLIENT_VERSION are temper's to define. The routing's "of another family or version" branch can't actually be reached, because the component answers 400 first.
10. temper's workspace `Cargo.toml` lists only `jig-protocol-charter` (:85) among these crates. It lacks jig-protocol-{store, worker-wire, engine-workers, worker-link, accounts, engine-clients, client-wire, config} and jig-test-store-files; the plan must add them.
