# Survey 01: temper's forge code against skein/docs/design/forge.md

Read-only survey. Line counts: "prod" excludes `#[cfg(test)]` tails and `src/tests*`; "test" is those. Graph
(`get_architecture`) was stale (lists temper.old), so counts come from rg/wc.

```
$ wc -l, grep -c '#\[test\]', awk over temper/crates/*/Cargo.toml   (per file; see sums below)
$ rg -n 'serde' --type rust temper skein jig ; rg -n 'std::|HashMap' temper-forge-forgejo/src
```

## (a) Module map (temper/crates/...)

| Path | prod | test | #[test] | temper deps | skein deps | external |
|---|---|---|---|---|---|---|
| temper-forge-forgejo (whole crate) | 7822 | 1478 | 84 | none | skein-lib, skein-json | none |
|  client/ (request encoders, answer decoders; 12 files) | 3433 | 0 | 0 | | | |
|  server/ (the fake forge's side; 12 files) | 2329 | 0 | 0 | | | |
|  decode/ (streaming walker) + document.rs | 545 + 487 | 0 | 0 | | | |
|  request, path, id, header, time, failure, limits, pin, delivery, lib | 1028 | 5 | 0 | | | |
|  tests.rs + tests/ (fixtures 215; 7 files) | 0 | 1473 | 84 | | | |
| temper-engine-domain-forge-client, only api.rs 1-415 (vocabulary) + locate.rs | 450 | ~31 | 2 | none | skein-lib | none |
|  same crate, rest (keep 1377, outbox 1079, api `accepts` 435, domain, bounds, boundary...) | 4124 | 2591 | 74 | | | |
| temper-engine-protocol-forge (whole crate) | 6314 | 3229 | 117 | forge-forgejo, domain-forge-client | lib, io, http, tls, json (dev: world) | none |
|  plans: plan 97, reads 793+331+150, writes 673, contents 391, actions 387, tables 91, reading 253 | 3166 | | | | | |
|  failure.rs (`classify`) 91; marker.rs 55; conventions.rs 67; hooks/hint.rs 32 | 245 | | | | | |
|  component: connection 746, component 581+322, limits 494, boundary 70, start 59, settings 48, grow 55 | 2375 | 49 | 0 | | | |
|  hooks.rs 181 + hooks/connection.rs 321 (listener, HMAC) | 502 | | | | | |
|  tests/ reads 27, writes 18, contents 23 fns (1817 lines): plans; connection 20, hooks 12 (505): component | | 3178 | 117 | | | |
| temper-forge-setup (reconcile machine, spec, upgrade) | 2851 | 1758 | 98 | forge-forgejo | lib, json | jig-protocol-config |
| temper-forge-setup-shell (host: command, files, exchange) | 1116 | 0 | 0 | setup, forge-forgejo | io, http, lib, shell, json | none |

Outside the crates (not counted above): tests/codecs 2906 lines 66 fns; tests/engine/protocol-forge 3110/53;
tests/client 2930/38; tests/forge-setup/reconcile 867/22; testing/temper-fake-forge-protocol; testing/temper-throwaway-forgejo.

forge.md 2.1 mapping: dialect, setup documents (client/setup, orgs, hooks, accounts), pin, deliveries = temper-forge-forgejo
already, whole. Vocabulary = api.rs + locate.rs. Plans, failure `classify`, keys' marker, user-agent/host helpers = protocol-forge.
The listener (hooks/), connections, start, tables' owner, limits stay for skein-forge-connection. keep/outbox/identity = skein-forge-client.

## (b) What is the owner's (symbols, files)

```
$ rg -n '"temper|b"temper|forgejo version' temper/crates/temper-{engine-protocol-forge,forge-setup,forge-setup-shell}/src
```

- Word: protocol-forge/marker.rs `HEAD = b"<!-- temper:key "`, `length(key) = 2*key + 21` (21 = 16 + 5; spec: 2*key + word + 15).
- User agent: protocol-forge/conventions.rs `USER_AGENT = "temper/0.0.0"`, used once, connection.rs `Connection::start`.
- Bot and names (setup; owner's per forge.md 2.2/6): forge-setup/spec.rs `ADMIN = b"temper-admin"`, `b"temper-runner"`;
  reconcile.rs:127,136,233 (`temper-admin`, `temper:bot` full_name marker); reconcile/tokens.rs:38-39 (`temper-api`, `temper-git`);
  check.rs:104,148; prepare.rs:319 (`temper-admin-<hex>`); setup-shell/reconcile.rs:65,72 (messages).
- Namespace: not in these crates (keys' namespace lives in domain-forge-client identity.rs `effect_key`, skein-forge-client's).
- Forgejo's own words are the dialect's (pin.rs, `b"forgejo"` hook type in client/hooks.rs:60, server/hooks.rs:43).

## (c) Audit against forge.md

Gap 1, "the start's two reads are the dialect's plans": NOT met. Today there is no plan type for them.
- protocol-forge/component.rs:557-561 `start_request` maps `Start::Checking` to `Request::User` and `Start::Settings` to
  `Request::ApiSettings` (both temper_forge_forgejo::request). component.rs:357-455 (`start_heard`, `drive_starts`) matches
  `Document::ApiSettings` and the user id by hand and builds `start::Heard::{User, ApiSettings, Status, Unanswered}`.
- start.rs (59 lines) holds the machine `Start::{Checking, Settings, Waiting, Ready, Stopped}` and `heard()`; backoff included.
- Spec (4.1, 4.2): two plans (identity read `GET /api/v1/user` -> id; page cap read `GET /api/v1/settings/api` -> `max_response_items` > 0),
  each `begin`/`heard` returning `Step::Send | Done`, in the dialect, uncounted in any call's cost. `plan::Plan` is `Read|Write` over `api::Op` only.

Gap 2, "the head check counts the user agent": NOT met. protocol-forge/limits.rs:260-269 sums `target_bytes + host.len() + token.len() + 192`
and compares with skein-http's `request` cap. The user agent is omitted (the owner's, of any length, per forge-connection.md 7.4). The doc on
`Unusable::Request` still says "(target, Host, token, agent)". The agent is a fixed 12 bytes today, so the omission is only latent.

Other differences:
- Plans take `&Limits`, `&Shared`, `&mut Tables` (plan.rs:49-65), not plain bounds (page size, rows, answer_bytes, body_bytes): verified.
- Plans hear `connection::Heard {Head{status, Fields}, Row, Done(Body), Failed(Failure)}` defined in the component (connection.rs:65): spec makes it the dialect's type; verified.
- `classify` is in protocol-forge/failure.rs (91), the refusal names in the dialect's failure.rs (42): split; moved together. Semantics against 5.4: not verified.
- Credential kinds: none. settings.rs `Forge.token: Secret`; rendering is inline in connection.rs (`token <t>`). Spec 4.1 and 8 want a kind and a render in the dialect: verified missing.
- Dialect `Limits` (+ `check`: zero, json.string < string_bytes/body_bytes, depth < 8): matches 4.1; verified.
- Pin: VERSION 17.0.0, RUNNER_VERSION 13.2.0, `names_forgejo`/`names_runner` match 4.3 literally; verified.
- Page size `min(rows, cap, answer_bytes / largest_row)` (limits.rs:299): matches 5.1; `largest_row` sizing not verified.
- Revision: first 8 bytes BE of SHA-256 of the kept body, else of length||updated nanos (reading.rs:66-80); goes through `skein_io::digest` (4 words) and
  hashes the kept bytes after the fact, not as the string streams: value verified, streaming not verified. Provenance (equal/later/else Unknown, Long -> Unknown): verified.
- Deliveries: 22 subscribed events and `/hooks/<n>` canonical, 16 bits (delivery.rs): verified. Listener verdicts 404/405/403/400/204 (hooks/connection.rs:137-209): verified;
  413 not verified. HMAC is the component's (hooks/connection.rs), as spec; dialect only decodes: verified.
- Marker arithmetic matches 6 for word `temper`; the word's 1-16 byte [a-z0-9-] rule is not checked anywhere: verified absent.
- Not verified: writes.rs details (CreateBranch 409 to Exists, UTF-8 refusal, TooLarge past 32 bits), time.rs range (1970..=2554), request sizing (counted-before-written).

## (d) serde

```
$ rg -c '\bserde' --type rust --type toml temper/crates/{temper-forge-forgejo,temper-engine-protocol-forge,temper-engine-domain-forge-client,temper-forge-setup,temper-forge-setup-shell}
```

Zero in all five crates (src and Cargo.toml). serde/serde_json appear only in other crates' tests and tools: crates/temper (116 lines in 12 files, tests and Cargo.toml),
tests/codecs (41 in 6 files), tests/one-process (16), tools/forgejo-conformance (30 in 18), testing/temper-throwaway-forgejo (3), tools/live-login (5), tools/git-transcripts (3).
skein-json (all no_std + alloc, forbid unsafe): `pub mod tokenizer` (Tokenizer, Limits, Event, Request) `pub mod writer` (Encoder, Limits, Refusal) `pub mod collector`
(Collector, Filter, Node) `pub mod document` and `pub use document::{Compact, Document, Kind}; token::Token`; paths are skein_json::{tokenizer, writer, collector, document}.

## (e) Users (use-site lines mentioning the crate / of them `use`)

- forge-forgejo: crates/temper 2/2, setup-shell 7/2, forge-setup (Cargo dep; src in lib), testing/fake-forge-protocol 38/16, throwaway-forgejo 2/2,
  tests/codecs 40/18, tests/engine/protocol-forge 1/0, tests/forge-setup/reconcile 12/10; protocol-forge (Cargo dep, most of its source).
- domain-forge-client: engine-domain 20/7, engine-domain-forge 21/16, engine-smith 22/5, protocol-store 21/14, service 10/3, protocol-forge (dep), tests/{client 1/0, codecs 1/1,
  conformance 10/4, engine/domain 10/2, engine/forge 14/14, engine/forge/client 8/8, engine/protocol-forge 13/9, system 5/1}, tools/forgejo-conformance 13/3.
- protocol-forge: crates/temper 3/3, service 18/6, setup-shell (dev) 2/0, tests/codecs 3/0, tests/engine/protocol-forge 13/15, tests/one-process 1/0, tools/forgejo-conformance 4/1.
- forge-setup: crates/temper 22/7, temper-box 5/3, setup-shell 12/16, tests/forge-setup/reconcile 12/13.

## (f) skein-lib and skein-json fit

- Signatures: skein-lib `sha256::{Sha256, digest, Hmac::{new, update, finish}, equal}` exist; used by the listener (component), not the dialect. skein-forge itself needs
  only `sha256::digest` (revision), today reached through skein-io, which skein-forge must not name. Also needed and present: `Secret`, `List`, `Writer`, `bytes`, `Decimal`, `Wall`/`Time`.
- Missing: nothing found for the dialect. The credential kind, the hint-table and `Heard` do not exist in skein yet (they are skein-forge's to add).

## (g) no_std feasibility

Already so: forge-forgejo has `#![cfg_attr(not(test), no_std)]`, `#![forbid(unsafe_code)]`, `extern crate alloc`, and no `std::`, HashMap, clock or io in production code.
It depends on skein-lib and skein-json only, both no_std + alloc. Moving the vocabulary: domain-forge-client is skein-lib only. The plans (protocol-forge) use skein-lib's
`Env`, `Queue`, `Token` and the component's types, but no std: not verified file by file.

## Blockers (to carry into the plan)

1. Plans and `Heard`/`Body` are entangled with the component's `Limits`/`Shared`/`Tables`/`connection` types.
2. The start's reads are not plans; both known gaps are in protocol-forge (component.rs, start.rs, limits.rs).
3. Word, agent and bot names are literals; the credential kind is missing; revision uses skein-io.
4. forge-setup depends on jig-protocol-config; forge.md 2.1 allows skein-lib, skein-json, skein-forge only.
5. api.rs mixes the vocabulary (1-415) with client-only `accepts` (473-850).
