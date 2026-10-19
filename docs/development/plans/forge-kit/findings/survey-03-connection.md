# Survey 03: temper-engine-protocol-forge as skein-forge-connection

Read-only, at origin/main of skein, temper, jig, smith. `K` is
`temper/crates/temper-engine-protocol-forge/src`. The graph was not used;
every name was found with `rg`.

```
$ wc -l K/*.rs K/*/*.rs K/*/*/*.rs
$ rg -c '#\[test\]' K
$ rg -no 'skein_(http|tls|lib|io|json)::[A-Za-z_:{}, ]+' K --glob '!tests*'
$ rg -n 'serde' temper-engine-protocol-forge temper-forge-forgejo temper-engine-domain-forge-client tests/engine/protocol-forge
$ rg -n 'temper-engine-protocol-forge' --glob Cargo.toml temper
```

## (a) Modules, dependencies, blockers

Total 9543 lines: 6365 production, 3178 test (117 `#[test]`); `no_std`.

| Files (production lines) | Sum | Belongs to (forge.md 2.1) |
|---|---|---|
| reads 793, reads/done 331, reads/size 150, reading 253 | 1527 | skein-forge (plans) |
| writes 673, contents 391, actions 387, plan 97 | 1548 | skein-forge (plans) |
| failure 91, marker 55, hooks/hint 32 | 178 | skein-forge (status map, key marker, delivery to hint) |
| component 581, component/calls 322, connection 746 | 1649 | skein-forge-connection |
| hooks 181, hooks/connection 321, start 59, tables 91 | 652 | skein-forge-connection |
| limits 494, grow 104, boundary 70, settings 48, conventions 67, lib 28 | 811 | skein-forge-connection (`limits` is shared with the plans, see c) |

Tests (lines/`#[test]`): tests.rs 355/17, connection 370/20, contents 708/23,
hooks 135/12, reads 658/27, writes 451/18, fixtures 501/0.

Cargo.toml: skein-lib, skein-io, skein-http, skein-tls, skein-json; temper:
temper-forge-forgejo (9300 lines; deps skein-lib, skein-json) and
temper-engine-domain-forge-client (7196 lines; dep skein-lib); jig, smith and
external: none. Dev: skein-world.
Blockers, all in the two temper crates that become skein-forge and
skein-forge-client (no jig, smith or other temper crate; `temper_engine_domain_forge`
is in a doc comment only):
- `temper_forge_forgejo`: `request::Request::{User, ApiSettings}`,
  `document::Document::{User, ApiSettings}`, `client::Decoder`, `decode`,
  `header`, `delivery::Delivery`, `failure::Refusal`, `id::ObjectId`,
  `time::http_date`; used by `connection`, `component`, the plans, `hooks`.
- `temper_engine_domain_forge_client`: `api::*` (Answer, Error, Op, Read,
  Write, Hint, Repository, ...) in every plan, `tables`, `failure`,
  `hooks/hint`; and `client::Limits` inside `limits::Shared.client`
  (`rows`, `answer_bytes`), which the connection must not depend on.

## (b) What is the owner's, and where it is read

| Value | Symbol and file | Read today |
|---|---|---|
| word | `marker::HEAD = b"<!-- temper:key "`, `marker::length(key) = 2*key+21` (`marker.rs`); `21` again in the `limits.rs` body check | hard-coded |
| user agent | `conventions::USER_AGENT = "temper/0.0.0"` (`conventions.rs:6`), pushed at `connection.rs:175`; `temper/src/forge.rs:259` names it too | hard-coded |
| namespace | not in this crate; `temper-service/src/forge.rs` `config()` hands the client the deployment id | owner |
| credential | `settings::Forge.token: Secret`, `user: u64`; `Webhooks.secrets: List<(u16, Secret)>` | files `api_token`, `webhook_secret` in `temper-service/src/startup/application.rs` (jig `config::File`) |
| settings | `Settings`, `Forge`, `Transport::{Plaintext, Tls}`: no word, no agent field | built in `temper-service/src/forge.rs` `settings()` |

The crate reads no file and no process environment; time comes from `env`.

## (c) Audit against forge-connection.md

| Design point | Code | Status |
|---|---|---|
| 3.1 word and agent handed in | neither is; both are constants | difference, verified |
| 3.2 word is 1 to 16 bytes of `[a-z0-9-]` | no check (no word) | difference, verified |
| 7.4 `body_bytes >= 2*op_bytes + word + 15` | `2*op_bytes + 21` in `limits.rs`, equal for `temper` | verified for `temper` |
| 7.4 head = target + Host + credential + **agent** + 192 | `limits.rs:262-271`: `target_bytes + host.len() + token.len() + 192` against `http(..).request`; the 12-byte agent is not counted. No test reaches `Unusable::Request` | the designer's gap, verified |
| 3.3 start runs the dialect's two plans, in no call's cost | `start_request` (`component.rs:557-562`): `Checking` gives `forgejo::request::Request::User`, `Settings` gives `Request::ApiSettings`. `drive_starts` (`:376-475`) calls `c.start(.., request, None)` at 436, no call. Heard: `Head{status != 200}` gives `Heard::Status`, `Document::User` gives `Heard::User{id}` (445-447), `Document::ApiSettings` gives `Heard::ApiSettings` (448-450) | verified; Forgejo types are wired in, the dialect's plans replace these arms |
| 3.3 stop on other user or a 4xx but 429; else back off, doubling to the max, a longer `Retry-After` wins | `start.rs:36-59` `heard`: same | verified |
| 8.5 close ends open calls with no terminal | `Request::Close` (`component.rs:172-188`): retires each waiting call, replaces `held` and `holders` with empty maps, `closed = true`, `hooks.close()`, `drive`; the io `Closed{owner}` handler finishes a holder's call only when `!closed` | verified; test `tests.rs:226` |
| 8.5 `closed` once, last | `component.rs:509-516` (`ended`, no connection, `hooks.closed()`) | verified; test `tests/reads.rs:649` |
| 6.3 signature 64 hex digits, 403 before the body | `hooks/connection.rs:313-321`, `is_ascii_hexdigit` | partly: uppercase passes the head, fails the HMAC after the body; case undecided in the design |
| 6.3 a head the server refuses gives a 4xx | skein-http `server::Rejection` 400/413/414/431/501/505, `Event::Refused` | verified on skein main |
| 7.3 shared values are plain numbers | `Shared{client: client::Limits, ..}`; plans read `shared.client.rows`, `.answer_bytes` | difference, verified |
| 7.2 token ranges, hooks owner `1<<59` | not re-checked | not verified |

## (d) serde and skein-http

serde and serde_json: 0 files in the crate, temper-forge-forgejo,
temper-engine-domain-forge-client and the world crate. JSON goes through
`skein_json::{tokenizer, writer}` (5 use sites).

skein main has no `skein-forge*` crate yet. Present, by path, with every
symbol the crate uses resolved:
- `skein-http/src/client.rs`: `Limits{request, head, headers, read, send}`,
  `largest_read`, `largest_room`, `worst_case`, `Request::{Call, Upload, Body,
  Discard, Close}`, `Event::{Response, Upload, Body, Done(Reuse), Failed,
  Closed}`, `Client::new`, `up`, `down`; `client/request.rs`: `Call`, `Body`,
  `Refusal`; root: `Header`, `Method`, `Version`.
- `skein-http/src/server.rs`: `Limits{head, headers, body, read, response,
  send}`, `worst_case`, `Request::{Next, Respond, Body, Discard, Reply, Close}`,
  `Event::{Call, Ended, Body, Reply, Refused, Done, Failed, Closed}`,
  `Rejection`, `Server`, `up`, `down`, `UP_MAX_OUT`, `DOWN_MAX_OUT`.
- `skein-tls`: `client::{Client, Limits, Request, Event, up, down, worst_case}`,
  `Name`, `Config`. `skein-lib`: `Secret`, `sha256::{Hmac, equal}`, `List`,
  `Map`, `Queue`, `Slab`, `Intake`, `Id`, `Token`, `Time`, `Wall`, `Duration`.
  `skein-io`: `Addr`, `Request`, `Event`, `digest::digest`.

## (e) Users (sites = lines naming the crate)

| Crate | Link | Sites, files | Symbols |
|---|---|---|---|
| temper-service | Cargo | 18, 11 | `Component`, `Event`, `Request::{Close, Offer}`, `Stopped`, `StartRead`, `Limits`, `MAX_OUT`, `settings::Transport`, `hooks::Webhooks`, `limits::{documents, worst_case, http}` |
| temper | Cargo | 3, 3 | `conventions::USER_AGENT`, `limits`, `StartRead`, `Stopped` |
| forge-setup-shell (dev), tools/forgejo-conformance, tests/codecs | Cargo | 2+4+3, 1+2+3 | `support/mod.rs`, `component.rs`, `hosted.rs`, forgejo tests |
| tests/one-process, tests/forge-setup/reconcile | Cargo, world | 2+3, 2+3 | `fixtures::fake_limits`, `pipe::{Cut, Pipe}`, `forge` |
| tests/engine/protocol-forge | the world | 21, 12 | `Component`, `Event`, `Request`, `Limits`, `Settings`, `Shared`, `Answered`, `hooks::Hinted` |

## (f) Tests

All 117 are in-crate (`src/tests.rs`, `src/tests/*`). The world crate is
`temper/tests/engine/protocol-forge` (`temper-engine-protocol-forge-world`):
3110 lines (src 1986, tests 1124, 53 `#[test]`). It depends on temper domain
crates, `temper-service`, the fake forge crates and `temper-engine-forge-world`.
- Moves with the crate: tests.rs, connection, hooks, fixtures, and
  reads/writes/contents (68), which drive a `Harness` over a `Component`;
  only `reading` is pure. skein-forge needs new pure `begin`/`heard` tests.
- Stays temper's: world suites that name the domain, service or fake forge;
  `tests/engine/forge`, `tests/fake-forge`, `testing/temper-fake-*`,
  `tools/forgejo-conformance`, `tests/codecs`.
- Likely split: the world's `fixtures` and `pipe` (two other worlds use them).
