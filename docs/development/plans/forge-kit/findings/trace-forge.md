# Design trace: skein forge.md 2-9, 11 and temper protocol/forge.md

Base skein b4cc6eb, temper 62d50e0b8. `K` = temper/crates/temper-engine-protocol-forge/src,
`D` = temper/crates/temper-forge-forgejo/src. Checked with rg/sed on the files, not the graph.

```
$ rg -n "Stage::|failure::|crate::(reads|writes|contents|actions)" K/{plan,reads,writes,contents,actions}.rs
$ rg -n "fn classify|fn expected|Drop::|Malformed::Bound|TooLarge" K/{failure,writes,contents,actions}.rs K/reads/done.rs
$ sed -n 295,330p K/limits.rs ; sed -n 165,175p K/connection.rs ; sed -n 300,325p K/hooks/connection.rs
$ rg -n "fn |#\[test\]" D/tests.rs D/tests/*.rs ; rg -c "#\[test\]" K
$ sed -n 60,100p D/client/start.rs ; rg -n "1_u64 <<|next_token" K
$ rg -n "enum Refusal" -A12 skein/crates/skein-http/src/client/request.rs
$ rg -n "temper:key|temper_engine_protocol_forge::" temper --glob '*.rs'   # outside K
$ ls temper/tests/codecs/tests ; sed -n '/^## 11/,/^## 12/p' skein/docs/design/forge.md
```

## 1. Kind (b): the design is silent or ambiguous on something observable

- **B1 Where the dialect's document tests live (11).** 11 wants transcript, seed and mutation tests and a fuzz
  target per decoder "in skein-forge, with no io"; transcripts are recorded by `tests/forge/conformance` (S07.2).
  Today they are `temper/tests/codecs` (7 suites, 55 tests; `fuzzy_forgejo.rs`, 3 sweeps) over
  `tools/forgejo-conformance/transcripts`: in no increment; they and the conformance cases hard-code `temper:key 01000001`.
  Options: (1) `skein-forge/tests/` (sweeps named `fuzzy_*`) reading transcripts moved at S01.1 into the conformance's
  output directory, with a neutral word; (2) a fifth world `tests/forge/codecs` (11 lists four worlds); (3) none until S07.2.
  Recommend (1).
- **B2 A write's answer whose list passes `inner`.** `Malformed::Bound`: read gives `InvalidAnswer`, write gives `TooLarge`
  (K/writes.rs:254; other malformed on a write is `Timeout`). 5.3: a malformed answer is `Timeout` on a write; 5.4: "past the
  component's bounds" is `TooLarge`. The client's outbox treats `TooLarge` in `Phase::Make` as a definite failure
  (temper-engine-domain-forge-client/src/outbox.rs:884) though the forge acted. Options: (1) `Timeout` on a write; (2) keep
  `TooLarge` and say so in 5.3; (3) skip the surplus elements. Recommend (1).
- **B3 Which `404`/`3xx` drop the repository's path.** 5.2 (l.712): a `404` on a file's side or a failing status's run/job
  "ends nothing: it is an answer"; 5.4: `404` is `Missing`, "the repository's path dropped". Code drops on every `404`/`3xx`,
  answer or not (contents.rs:136, actions.rs:122, `failure::classify` `Drop::Path`); the next call pays a resolve (+1 request).
  Options: (1) say every `404`/`3xx` drops; (2) drop only when it ends the call; (3) never on `Job` or side reads.
  Recommend (1) (a move changes no behaviour).
- **B4 `File` and `Job` when `max_bytes` exceeds the room.** 3.2: both "cut to `max_bytes` and say when they cut"; 5.3: raw
  contents are read "within what is left of `answer_bytes`". `Job` cuts at the room and sets `truncated` (actions.rs
  `raw_room`); `File` answers `TooLarge` when more comes and room < `max_bytes` (contents.rs `raw`). Options: (1) `File`
  cuts and says so like `Job`; (2) `TooLarge` for both; (3) admission: `max_bytes` + path/step name <= `answer_bytes`.
  Recommend (1).
- **B5 Absent or invalid answer headers (4.2 names the fields, not their absence).** No `Date` gives `now` 0 on `Items`;
  `429` with no/invalid `Retry-After` gives `after` 0; a date-form `Retry-After` without `Date` uses the component's
  clock; no/invalid `X-HasMore` gives `more` false on `PullFiles`; invalid `Content-Range` gives no `truncated`.
  Options: (1) document all defaults; (2) a missing `Date` on `Items` is a malformed answer, the rest documented.
  Recommend (2): a `now` of 0 feeds the keep-up `since`.
- **B6 The signature header's shape (7 says "hexadecimal").** Code: absent or not 64 ASCII hex digits gives `403` from the
  head, body unread; 64 digits of the wrong value gives `403` after the body; upper-case hex passes the head, then fails
  the lower-case compare. Options: (1) state: a header that is not 64 lower-case hex digits is "no signature";
  (2) accept either case, compare after lower-casing; (3) every bad header is "wrong" (body read first). Recommend (1).
- **B7 Success statuses and the unlisted status.** 5.2's table names no success status; code `expected()` (writes.rs):
  `201` for CreateIssue, Post, Edit, Close, Reopen, Status, CreateBranch, OpenPull and POST reviewers; `200` for Review,
  Merge, Update; `204` for DeleteBranch and DELETE reviewers. Any other status, a `2xx` included, is `Timeout` on a write and
  `InvalidAnswer` on a read (5.4 lists neither). Options: (1) add a status column to 5.2 and a 5.4 row; (2) accept any
  `2xx`. Recommend (1).
- **B8 `GET repositories/{id}` answering another id.** Reads keep the row under the answer's id; writes give `Timeout`
  (reads/done.rs `repository_done`; 5.1 is silent). Options: (1) `InvalidAnswer` for reads too; (2) document both.
  Recommend (1).
- **B9 The page size's row measure (5.1, 5.3, forge-connection.md 7.4).** "The number of the listing's largest rows
  `answer_bytes` holds" has no measure; K/limits.rs:306 `largest_row` sums `size_of` of the vocabulary row and its owned
  bytes, so the wire `limit` and the start refusal `Unusable::PageSize` vary with pointer width. Options: (1) name the
  client's `answer` measure in 3.7 as the one rule; (2) a platform-free count. Recommend (1).
- **B10 Bytes of the user agent and the credential.** The head check counts sizes only; skein-http refuses a header value
  with CR, LF or NUL (`client::request::Refusal::Value`), which the component maps to `Unseen`, so a token holding an LF
  makes every call `Unavailable` at run time (K/connection.rs:430). Options: (1) start refuses control bytes in token, agent
  and the Host (a new `Unusable`); (2) leave it; (3) the dialect's credential render returns a `Result`. Recommend (1).

## 2. Kind (a): the design is clear; the plan adds it to the increment

- **S01.1 (dialect, 84 tests).** Add: `Limits::check` direct tests (`Zero` per field, `TokenizerString`, `Depth` for json and
  writer; only `Depth` is reached, K/tests.rs:274); the stack's depth at the limit and one past (no test in D/tests*); a
  string cut at every byte (11); golden bytes for every request (the 84 hold hooks, orgs, accounts, tokens, edits, escapes);
  each server encoder met by the client decoder and the reverse (found for deliveries and the user only).
- **S01.2 (vocabulary).** A test that no plan answers `Busy` (3.4), added in 01.3/01.4; counts 19 reads, 13 writes, 24
  answers, 18 errors need no test.
- **S01.3 (read plans, marker).**
  - Word: check `[a-z0-9-]` and 1..=16 (6); the outline says only 1-16; tests for empty, 17, upper-case, space, `:`.
  - Arithmetic `2*key + word + 15` replaces `21` in marker.rs `length` and K/limits.rs:189 (S03.1); tests with words of 1
    and 16 bytes and a key at `op_bytes`; "another word has no key" (none today); odd/upper-case/no-LF exist (tests.rs:36-47).
  - Pure `begin`/`heard` tests per call (11 Plans): the 68 plan tests drive a `Harness` over a `Component`, so they move at
    S03.1; the pure ones to write are page size = smallest of three, `%2F`, 2554, each 5.4 row by status and message, first
    failure ends the call with the cost counted, `Unseen`/`Seen`, tables filled, a row dropped, LRU evicts the longest unused.
  - `UserSearch` (`GET /api/v1/users/search?uid=<id>&limit=1`, in forge-connection.md:176 only) for Permission,
    Collaborators, SetReviewers; not found is `Missing`.
  - Revision: the code hashes the kept body after the fact (`skein_io::digest`); the value equals the streamed one for a
    kept body; a long body's `length || updated_at` layout is unspecified: golden-test the code's layout.
- **S01.4 (write plans, contents, actions, hints).** Pure tests for: each write's body and status; `409` `head out of date`,
  `405` variants and `409` update conflict by message; `OpenPull`'s three reads counted and mapped to `Unavailable`;
  `SetReviewers` order (add, then remove); `Stale` on a `PullFiles` head; `Job` `MissingJob` on head, run, attempt;
  step name past `string_bytes` carried empty (found in D/tests.rs). `hint` for the 22 events and 13 headers, a tag, a
  branch deletion, a job end, a repository id past 32 bits, a body not kept whole (no key): today only through
  `tests/hooks.rs` (S03.1).
- **S01.5 (credentials, start plans).** `Authorization: token <t>` render; the value in no `Debug`; `max_response_items`
  0 refused; the start plans in no call's cost.
- **S03.1/S03.2.** `Unusable` for a word off the charset (6, "checked at start"); the head-check test (outline has it).
- **T01.3.** A temper test that its word makes `<!-- temper:key <hex> -->\n` (21 bytes of overhead) and its agent is
  `temper/0.0.0` on the wire: temper's doc 4 pins them, and the kit's tests will use other words.
- **S07.2.** The conformance's cases (`tools/forgejo-conformance/src/{cases,deliveries,facts}.rs`) take the word from the owner.

## 3. The surveys' "not verified"

- survey-01 `classify` against 5.4: TRUE with notes B2, B3, B7 (K/failure.rs `classify`; every 5.4 row found, `3xx` and
  `423` included, `403` on DeleteBranch `Protected`, `MergeStyle` drops the row).
- survey-01 `largest_row`: TRUE as a formula, K/limits.rs:306 (see B9). Revision streaming: FALSE (after the fact).
- survey-01 CreateBranch `409` to `Exists`, UTF-8 refusal, `TooLarge` past 32 bits: TRUE (K/writes.rs `heard`, `validate`;
  K/reads/done.rs `repository_done`). Time range 1970..=2554: TRUE (D/time.rs).
- survey-01 request sizing counted before written: TRUE for D/client/start.rs `encode` (length, `Refusal::TooLong` past
  `target_bytes`, then `Writer::new(length)`); the other families share `Encoded`/`Refusal`, not read file by file.
- survey-01 plans' no_std: TRUE (K/lib.rs `#![cfg_attr(not(test), no_std)]`, no `std::` outside tests); `reading.rs` uses
  `skein_io::digest`, the one name skein-forge must not keep.
- survey-03 `413`: TRUE (skein-http `Rejection::BodyTooLong`, server.rs:357-367; K/hooks/connection.rs `hook_http.body`).
- survey-03 "7.2 token ranges, hooks owner `1<<59`": the design names none (7.2 is limits); code K/hooks.rs:21,147-149 and
  K/component.rs:519-521 (connection tokens below `1<<60`): not a rule of this document.
- survey-03 signature case: TRUE (B6).

## 4. Names the design gives

- skein-forge: `api`, `locate` (modules); `Read`(19), `Write`(13), `Op`, `Answer`(24), `Error`(18), `Hint`, `Change::{Item,
  Branch, BranchMoved{branch,head,actor}, Commit}`, `Repository{forge,repository}`; plan `begin`/`heard`, `Step::{Send,Done}`,
  `Heard::{Head,Row,Done,Failed}`, `Failure::{Unseen,Seen}`, ranges head/tail, `classify`; dialect `Limits{json,writer,
  string_bytes,body_bytes,inner,target_bytes}` + `check`; `Delivery::{Issue,Pull,Comment,Review,Push,Ref,Job}`, job ends
  `Success|Failure|Cancelled|Skipped`; pin `VERSION` 17.0.0, `RUNNER_VERSION` 13.2.0, `names_forgejo`, `names_runner`;
  headers `X-Forgejo-Event`, `X-Forgejo-Signature`; path `/hooks/<n>`; marker `<!-- <word>:key <hex> -->\n`, bot `<word>:bot`.
- skein-forge-client: `effect_key(namespace, identity, max)` (version byte 1, namespace length u16 BE, namespace, identity).
- skein-forge-connection: limits `forges`, `token_bytes`, `repositories`, `users`, `connections`, `idle`, `head_bytes`,
  `headers`, `body_bytes_in`, `depth`, `chunk`, `deadline`, `start_backoff(_max)`, `hooks`, `hook_head_bytes`,
  `hook_body_bytes`, `hook_deadline`, `hook_secret_bytes`; shared `calls`, `lifetime`, `rows`, `answer_bytes`, `op_bytes`.
- testing: `testing/skein-fake-forge-{domain,protocol}`, `testing/skein-fake-peers` (forge peer),
  `testing/skein-throwaway-forgejo`, `tests/forge/{machine,protocol,client,setup}`, `tests/forge/conformance`.
- tests: no names for skein-forge tests (11 gives categories); facts `component-read-items` .. `component-write-delete-branch`,
  `component-read-pull-for-open-after-closed`, `component-write-open-pull-no-changes`, `component-start-ready`,
  `component-costs-equal-requests-sent`, `component-keys-found-on-read-back`, plus the `delivery-*` slugs of 9.

## 5. The outline against the design

- **C1 01.3 cannot land before 01.4.** reads.rs:217,230 builds `contents::Plan` and `actions::Plan` (01.4); reads.rs:288,302
  call `failure::read`/`transport` (01.4's classify); plan.rs:34,46 names `writes::Plan` (01.4); writes.rs:261 and
  contents.rs:162 call `reads::keep_document`; failure.rs imports `connection::{Failure, Fields}` (01.3's `Heard`). Land
  `Heard`/`Body` and `failure` first, then the plans together (or make 01.3 and 01.4 one increment).
- **C2 `conventions` in skein-forge (01.3).** `host()` and `USER_AGENT` build the component's headers (connection.rs:168-175,
  settings.rs:26) and `host()` calls `marker::hex`; 2.1 gives skein-forge no Host rule. They belong with S03.1, `USER_AGENT` dropped.
- **C3** `tests/codecs` and the transcripts are in no increment (B1).
- No wrong crate or dependency found: S02 on the vocabulary only, S03 off the client, `skein-forge` on skein-lib and
  skein-json once `skein_io::digest` is swapped; temper's doc 10 keeps its protocol world and cost checks, as T06 does.
