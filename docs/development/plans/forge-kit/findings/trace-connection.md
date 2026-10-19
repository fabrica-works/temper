# Design trace: forge-connection.md 2-10 and temper protocol/forge.md

Read only. `K` = `temper/crates/temper-engine-protocol-forge/src`; `W` = `temper/tests/engine/protocol-forge`.
Heads checked out: skein d120820, temper 27995fb75 (the outline's bases are b4cc6eb and 62d50e0b8; see Q4).
The graph was not used; every name was found with `rg`.

```
$ rg -n '<< ?(59|60)|OWNER|next_token' K                  # token ranges
$ rg -n 'Unusable::Request|192|USER_AGENT|fn host' K       # head check, agent, Host
$ rg -n 'tables::|mod tables' K ; sed -n 1,30p K/tables.rs # who uses the tables
$ rg -n 'is_ascii_hexdigit|close: status|fn verified|fn signature' K/hooks
$ rg -n '#\[test\]' K W temper/tests/fake-forge            # which rules have a test
$ rg -n 'ListenerLost|webhook_secret' temper/crates/temper-service/{src,tests}
$ sed -n 1,500p skein/docs/design/forge-connection.md      # sections 2-10 re-read
$ rg -n 'Refusal::Value|fn status' skein/crates/skein-http/src
$ ls skein/tests/tls skein/tests/heap                      # patterns for the TLS world, allocator test
```

## Q1. Rules neither built nor tested

### (b) Design questions (silent or ambiguous on something observable)

- **B1. A header value with a control byte (3.1, 3.2, 7.4).** The user agent is "the owner's, of any length" (7.4)
  and the credential is checked only for empty or too long (3.2). A CR, LF or NUL in the agent, the Host or the
  token makes skein-http's client refuse the request (`Refusal::Value`, `client/request.rs`); K/connection.rs maps
  `http::Error::Refused` to `Failure::Unseen`, so the forge never starts and no refusal names the cause.
  Options: A, a named refusal at start (`Unusable::UserAgent`: empty, or a byte below 0x20 other than tab, or 0x7f;
  the same for a token) with one-below/at tests; B, leave it as the owner's duty (9) and say so; C, check the
  agent only. **Recommend A** (the word is checked, so is the agent: both are owner values that reach the wire).
- **B2. Uppercase hex in `X-Hub-Signature-256` (6.3).** The head check takes 64 `is_ascii_hexdigit` bytes
  (K/hooks/connection.rs signature()); uppercase passes the head, the body is read, the compare fails: 403 after
  the body, where 6.3 promises 403 before it. Options: A, keep and add a test that names it (403, after the body);
  B, require lowercase in the head check (403, body unread); C, compare case-insensitively. **Recommend A**
  (Forgejo sends lowercase; a wrong-case digest is a wrong signature; the design only states the 64 digits).
- **B3. Who builds `Host` (3.1, forge.md 5.1).** 3.1's list of owner values omits it; forge.md 5.1 derives it
  from the TLS server name or the address. Today `conventions::host(address, server_name)` is called by
  temper-service (forge.rs:56), crates/temper (forge.rs:229-259) and the component. Options: A, the owner hands
  `host` (or the component derives it by `skein_forge::conventions::host`), 3.1 amended and tested for the default
  port and IPv6; B, the kit derives it always, which loses a plaintext forge's name. **Recommend A.**
- **B4. 6.3 "a head the server itself refuses | a `4xx`".** skein-http's `Rejection::status()` also gives 501 and
  505 (survey 03 calls the row "verified"; it is true for 400, 413, 414, 431 only). Options: A, amend 6.3 to "the
  server's own answer"; B, map 501/505 to 4xx in the listener. **Recommend A.**
- **B5. The component's token space (2, 7.2).** 2 says the owner gives io its own tokens and keeps the component's
  in each; nothing says the component's tokens must be below 2^60 (K asserts it), nor how the component splits
  them: connections 1.., the listener's owner `1<<59`, hook connections `1<<59 + 1..` (K/hooks.rs:21,41,149;
  K/component.rs:519-521). Options: A, state in 2.1 that the component's tokens are below 2^60 and the owner's
  prefix sits above; B, an owner-handed limit. **Recommend A.** Code nit for S03.2: the connection assert should be
  `< 1<<59`, or a connection token can meet the listener's owner (never reached: 2^59 connections).
- **B6. (Low) `Retry-After` at start is uncapped (3.3; temper-forge-forgejo header.rs:43).** A 429 with an hour
  wins over `start_backoff_max`. Options: A, honour it as the design says; B, cap at `start_backoff_max`.
  **Recommend A** (3.3 says "wins"). Also (optional): a delivery with no `X-Forgejo-Event` gets 204 after
  verification and no hint; say so in 6.3.

### (a) The design is clear; add to the outline's increment

- **S03.1 (the component moved).** Settings gain the word, the user agent, and each forge's credential kind
  (3.1; K/settings.rs has none). Add: the 3.2 refusal for a word outside 1-16 bytes of `[a-z0-9-]` (new
  `Unusable::Word`) with tests at 0, 1, 16, 17 and a byte outside the set; `body_bytes >= 2*op_bytes + word_len + 15`
  (K/limits.rs: `+ 21`) with one-below and at; the credential rendered by the dialect (K/connection.rs:
  `Authorization: token <t>`, today Forgejo's); `Shared` as plain numbers (`rows`, `answer_bytes`, `op_bytes`,
  `calls`, `lifetime`, `name_bytes`, `repositories`, `io`); the start reads named neutrally (`StartRead::{User,
  ApiSettings}`) with the dialect's plans; `RepositoryEntry` and `UserEntry` (K/tables.rs, `pub(crate)`; sized by
  K/limits.rs:383) made visible where `worst_case` can size them, since the tables move in S01.3 and the component
  keeps them (4.2).
- **S03.2 (the head check and the trace's differences).** Head = `target_bytes + Host + credential + user agent +
  192` (K/limits.rs:262-271 omits the agent); a test reaching `Unusable::Request`. Then 7.4: each check has one refusal test
  (K/tests.rs:115-131, 239-274; tests/hooks.rs:8-18), none paired with its at-bound acceptance (10: "one below
  and at"); no test reaches `Unusable::Forges`, `Listener` or `Request`; `Token` is tested empty, not past
  `token_bytes`; `Documents` only for depth; `deadline == lifetime` and `hook_secret_bytes == 32` accepted: none.
  Also: `page_size` and `largest_row` for all 7 `Listing`s (only
  `Items` is tested); a call on an unconfigured forge ends `Unavailable` at once (K/component/calls.rs ~84, no
  test); two-forge tests (every fixture has one forge): idle connection of one forge closed for a call to another
  (calls.rs `assign`, `other` branch), a stopped forge does not stop the other; backoff doubling to
  `start_backoff_max` and a longer 429 `Retry-After` winning (start.rs `heard`: only the status arms are tested);
  63/64/65 digit signatures; the B5 assert.
- **A new increment, "S03.3 tests/forge/machine" (after S03.1; before S06).** 10 asks for the component's machine
  world, plaintext AND TLS forge, with a fault at every byte, and the listener against scripted deliveries with a
  fault at every byte; testing.md:246 and forge.md 11 name it `tests/forge/machine`. Today: `W/tests/{machine,
  hooks,fuzzy_machine,fuzzy_hooks}.rs` and `W/src/{machine,hooks,peer,pipe}` (about 37 tests, plaintext only,
  listener a split sweep, not a fault at every byte). To add: a TLS forge on skein's `tests/tls` fixtures (rustls
  in-memory server, as `tests/tls/memory.rs`); deadlines in each phase (start read, call, hook request); owner
  `Close` in every state crossed with a racing terminal, with no spin and `closed` once, last (8.5; the unit
  tests cover two states); the worst case against the counting allocator (7.5; no test exists; pattern:
  `skein/tests/heap`).
- **T03 (temper's side).** Users to rewire beyond temper-service: crates/temper (`conventions::USER_AGENT`,
  `limits::{http,documents}`, `StartRead`, `Stopped`), forge-setup-shell, tools/forgejo-conformance, tests/codecs,
  tests/one-process, tests/forge-setup/reconcile (survey 03 e). temper-service hands the word `temper` and the
  agent `temper/0.0.0` from one constant; `small()` (forge.rs, hard-codes `+21`) derives from the word
  (`6 + 15`); `problems()` maps the new `Unusable` variants to config paths; `tag()` keeps the FORGE prefix.
- **T03 or T06 (temper's own rule).** temper forge.md 10, busy pass: the item-read bound is not asserted (the
  protocol world's busy test asserts only listing pages `<= 7`). The idle, history-free and honest checks exist.
  Temper's other rules in forge.md are built and tested and stay: `webhook_secret` with no `webhooks` refuses
  the start (temper-service `tests/checks.rs:247`); a listener that fails stops the engine (`tests/forge.rs:138`).

## Q2. Surveys' "not verified"

- **"7.2 token ranges, hooks owner `1<<59`": TRUE for the code.** K/hooks.rs:21 `OWNER = Token::new(1<<59)`;
  hook connections from `OWNER+1`, assert `< 1<<60` (hooks.rs:41,149); component tokens from 1 (component.rs:89,
  `open_start` 519-521), assert `< 1<<60`. temper-service `tag()` ORs the FORGE prefix above bit 60 and keeps
  the component's token. But the design's 7.2 states no token range (see B5), so the survey's check has no
  design side; and the ranges overlap in theory (nit above).
- **Survey row "6.3 a head the server refuses gives a 4xx: verified" is partly FALSE:** `Rejection::status()`
  (skein-http `server.rs`) gives 400, 413, 414, 431, 501 and 505 (B4).
- Survey row "7.4 head ... agent": confirmed (K/limits.rs:262-271, no `Unusable::Request` test in K or W).

## Q3. Names the design gives (path: names)

- `skein-forge-connection` (no_std+alloc; deps skein-lib, -io, -http, -tls, -json, skein-forge; not the client):
  `Component` (`begin`, `down`, `up`, `fire`, `next_deadline`), `MAX_OUT` = 16 above and 16 below, `Request`
  (call, refused, refused hint, returned, offer, close), `Event` (answered, hint, listener lost, stopped, closed),
  `Answered`, `Stopped`, `StartRead`, `Settings`, `Limits`, `Shared`, `Unusable`; `settings::{Forge, Kind,
  Transport}`; `hooks::{Webhooks, Hinted}`; `limits::{check, documents, http, hook_http, hook_documents,
  worst_case, page_size, largest_row, Listing, Stacked}`; `start::{Start, Heard, heard}`; `grow`; `boundary`.
- `Limits` fields (7.2): forges, token_bytes, repositories, users, connections, idle, head_bytes, headers,
  body_bytes_in, depth, chunk, string_bytes, body_bytes, inner, target_bytes, deadline, start_backoff,
  start_backoff_max, hooks, hook_head_bytes, hook_body_bytes, hook_deadline, hook_secret_bytes, tls.
  `Shared` (7.3): calls, rows, answer_bytes, op_bytes, lifetime, name_bytes, repositories, io.
- States: forge connection (8.1), call (8.2), start Checking, Settings, Waiting, Ready, Stopped (8.3), webhook
  connection (8.4). Reads: `Read::Locate { owner, name }`, `Answer::Located { repository }`. Listener answers:
  404, 405, 403 (head order), 413, 403 (signature), 400, 204 (keep-alive). `skein_lib::sha256::{Hmac, equal}`.
- `skein-forge` (no_std+alloc, lib-only): `api`, `locate`, `forgejo`, `conventions::{host, USER_AGENT}` (agent from
  the owner), `marker::{HEAD, TAIL, length, encode, decode, hex}` (word from the owner), `tables::{Repository,
  Tables}`, `delivery::{hook_target, hook_forge, EVENT_HEADER, SIGNATURE_HEADER}`; plans `begin`/`heard`, `Send`/`Done`,
  `Heard`/`Body`; credential kinds (forge.md 8).
- Test crates: `tests/forge/machine` (the component's and the listener's worlds + worst case),
  `tests/forge/protocol`, `tests/forge/{client,setup,conformance}`. Temper keeps `tests/engine/protocol-forge`.
- Test names the 10 section asks for (design gives none by name): signature at 63, 64, 65 digits; each limit check
  one below and at; each listing's page size; faults at every byte, plaintext and TLS; counting-allocator worst case.
- Temper's side: `temper-service/src/forge.rs` (`settings`, `shared`, `small`, `Layer`, `problems`, `tag`,
  `indexed`, `event`, `hinted`, `back`), `tokens::FORGE`; the word `temper`, agent `temper/0.0.0`, bound 6+15=21.

## Q4. Contradictions in the outline

- **05.3 gives `tests/forge/machine` to "temper's tests/fake-forge memory tests".** The design (testing.md:246-250,
  forge-connection.md 10, forge.md 11) makes it the component's and the listener's machine world; the fake
  forge's memory tests run "in binaries of their own" (fake-forge.md 8). S06 lists only protocol, client and
  setup. No increment builds the world: see S03.3 above (the plain-value `Shared` in S03.1 is what makes it movable).
- **S03.1 "117 tests move".** 68 of them (reads, writes, contents) drive a `Harness` over a `Component`; they cannot
  move with S01.3/S01.4 (skein-forge has no component) and stay in temper until S03.1. Then S01.4 (writes,
  contents, actions) has no pure `begin`/`heard` tests in its text (01.3 has); add them, or those rules are
  tested only through the component.
- **Survey 03 files `tables` (91 lines) under the connection; the outline moves it with the plans (S01.3).** The
  outline is right (the plans read and fill rows: forge.md 4.2, connection 4.2); S03.1 must not move it again.
- **T03 names only temper-service as a user.** Seven more crates use the component (Q1, T03).
- **The outline's bases (skein b4cc6eb, temper 62d50e0b8) are not the checked-out HEADs** (d120820, 27995fb75);
  line numbers above are the HEADs'.
- No wrong dependency: S03 needs S01 and T01.4, not the client; T03 needs T02.1 for the client's numbers. The
  start's two plans (S01.5) come before the component moves (S03 needs S01), as 3.3 wants.
