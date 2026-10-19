# The forge's protocol layer

Provisional, 2026-10-03, revised 2026-10-18. How the engine reaches the
forge, on skein's forge kit: what temper's forge component and forge
connector do with it, and what the kit leaves to temper. The kit is
skein's (`forge.md`, `forge-connection.md`, `forge-client.md`,
`forge-setup.md`, `fake-forge.md`): Forgejo's API over HTTP and JSON, the
keys' markers written at the head of what the engine creates, the
webhooks Forgejo sends back to a listener on loopback, the client, the
component, the setup and the fake forge. The overview is README.md
(sections 5 and 6); the vocabulary carried is skein's (`forge.md`,
section 3), and the forge client's use of it is `../domain/forge.md`'s
(sections 5 and 6); the machines are skein's (skein's `http.md`,
`json.md`, `tls.md`). What others owe this layer is in section 12, what
temper relies on of Forgejo in section 13, and what is open, GitHub
among it, in section 14.

## 1. In one page

- **The component and the client are skein's, owned by temper.**
  `skein-forge-connection`, the forge component, runs as a child of the
  engine's protocol layer. `skein-forge-client`, the forge client, is
  owned by temper's forge connector (`temper-engine-domain-forge`).
  Forgejo's documents and call plans, the keys' markers, the deliveries,
  reconcile and the fakes are skein's too (skein's `forge.md`, section
  2). This document holds what the kit leaves to its owner.
- **Translation only, and no retries.** Each call of the forge client (a
  `Read` or a `Write`) is answered by exactly one `Answer` or `Error`,
  made of one or a few HTTP exchanges on one connection. A call goes out
  once. Its answer says how many requests it cost, so the client's
  budget charges what the forge actually served (skein's
  `forge-connection.md`, section 1; `forge-client.md`, section 6).
- **temper's routing hands the component every value** (section 2): the
  component reads no file. The values are temper's configuration's, with
  two of temper's own: the word `temper` and the user agent
  `temper/0.0.0`.
- **Repositories by the forge's own id; paged, bounded, and decoded as
  it streams.** Forgejo's conventions are skein's (`forge.md`, section
  5); the paths and the users the component keeps are
  `forge-connection.md`'s (section 4). Adoption first locates a
  repository by its owner and name (skein's `forge.md`, 5.2).
- **The engine's marks are keys' markers only** (section 4). A creation
  carries its key at the head of its body. The word is `temper`, and the
  key holds the deployment's id as its namespace. The marker's form is
  skein's (`forge.md`, section 6). Nothing else of the engine's is
  written into the forge.
- **The API token is the component's alone.** temper's binary reads it
  from its file and the routing hands it over at start. It is never
  granted and never stored. The first exchange with each forge checks
  that the token belongs to the configured user, and a wrong one stops
  the start (skein's `forge-connection.md`, section 3; credentials.md,
  section 5).
- **Webhooks for latency, polling for liveness** (section 5). Forgejo's
  deliveries reach a listener of the component's own, on loopback, at
  `application.webhooks`'s address: verified as they stream, answered
  first, each hint naming what caused it (skein's `forge.md`, section 7;
  `forge-connection.md`, section 6). Polling stands as it is and finds
  what a delivery lost, refused or never sent told of; a forge on
  another host polls alone, since deliveries over TLS need skein's TLS
  server side (setup.md, section 15).
- **The load rules' values are temper's** (section 6): the rules are
  kept in skein's crates, and `../domain/forge.md` holds the values.
- **The fake forge, the throwaway and the conformance are skein's**
  (`fake-forge.md`). temper's worlds run temper's stories over them and
  count its costs (section 10).
- **What temper relies on of Forgejo is recorded** by skein's
  conformance against a real Forgejo v17.0, each behaviour a named fact
  (skein's `forge.md`, section 9). temper requires that release: one
  patch release of it is pinned (skein's `forge.md`, 4.3), and no other
  version is served.

## 2. In the engine

```
skein-forge-client                  the forge client: Read, Write, Answer, Error, Hint (skein-forge's vocabulary)
  ▲  the root: Request::Forge; Event::Forge (the client's Answered, the forge's Hint)
the routing (temper-service)        io ↔ components ↔ root; hands the component its values
  ▲
skein-forge-connection              calls, connections, repositories' paths, users, each forge's start; the webhooks' listener: verified deliveries as hints
  ▲  documents, call plans
skein-forge                         Forgejo's requests and answers, both sides; deliveries' payloads; the markers; the org hooks' and the setup's documents
  ▲  tokens
skein-json
  ▲  body stream
skein-http (client; a server for webhooks, plaintext on loopback)
  ▲  plaintext
skein-tls (client, for a forge whose transport is TLS; later, a server for webhooks from another host)
  ▲  ciphertext
skein-io socket
```

- **The kit's crates** are skein's (`forge.md`, section 2). `skein-forge`
  depends on skein-lib and skein-json and holds nothing of temper's
  domains; `skein-forge-client` depends on skein-lib and `skein-forge`'s
  vocabulary; `skein-forge-connection` depends on the crates below it in
  the diagram. `skein-forge-setup` is reconcile's: temper's setup shell
  hosts it (setup.md, section 11).
- **The forge component,** `skein-forge-connection`, is a child of the
  engine's protocol layer, beside jig's components, with its own limits,
  worst case, deadlines and most outputs per entry point; the routing
  carries what passes between it, io and the root, and decides nothing
  (README.md, sections 5 and 6; jig's `protocol/service.md`, section 2).
  One component serves every configured forge. What it holds is skein's
  (`forge-connection.md`, sections 3 to 7): the calls in flight, the
  forges' connections, each forge's start, the repositories' paths and
  the users, the translation between the forge client's vocabulary and
  the documents, and the webhooks' listener.
- **What the routing hands the component.** The component reads no file
  (skein's `forge-connection.md`, section 3). temper's binary reads the
  files, and the routing, `temper-service`, hands over the values. For
  each entry of `application.forges` (configuration.md, section 3):
  - the forge's number, and its kind, Forgejo, which names the dialect;
  - its API's address and transport: plaintext, or TLS with its server
    name and its trust (the machine's roots when `trust` is null);
  - the API token, a credential of the kind token, its bytes from
    `api_token`'s file;
  - the user the token belongs to: a number, or the number a file holds
    (setup.md, section 8);
  - the webhook secret's bytes, from `webhook_secret`'s file, when the
    forge has one.

  And `application.webhooks`, the listener's address, when configured.
  And temper's two values of its own: the word `temper`, which marks its
  keys and its bot (section 4), and the user agent, `temper/0.0.0`. And
  the limits: temper's `limits` member (configuration.md, section 7),
  and the values the component shares with the forge client (section
  12).
  The setup shell gives reconcile the same word (setup.md, section 5),
  and its requests carry the same user agent. The files are private files
  (configuration.md, section 6). The component reads no other member: a forge's `git` base
  is the routing's, for runs' remotes (channel.md, section 4), and its
  `identity` names a git identity, which the component never holds.
- **What crosses to the root** is the root's boundary: a
  `Request::Forge { call, repository, op }`, which the routing hands to
  the component, ended by exactly one `client::Event::Answered { call,
  cost, result }`, which the routing hands the root as `Event::Forge`
  (the forge's `Event::Client`); and the forge's `Event::Hint { hint }`,
  which no request causes: a verified delivery's (section 5;
  `../domain/forge.md`, 5.2).
- **Room, and the drain,** as the component keeps them (skein's
  `forge-connection.md`, section 2) and as jig's components keep them
  (jig's `protocol/service.md`, section 2; jig's `domain/engine.md`,
  6.1): an answer or a hint the root cannot take yet is held and offered
  again, at most one per call and one per webhook connection, and
  nothing more is read off its connection meanwhile. One the root
  returns during its drain (its `Request::Busy`) is never offered
  again: it goes when the component closes, and the next process reads
  afresh and settles its outbox (`../domain/forge.md`, section 16). The
  routing closes the component after the root's drain (jig's
  `protocol/service.md`, section 5); calls in flight end with their
  connections, and no answer goes up once the root has stopped.
- **A start that fails for good ends the engine,** exit status 1,
  naming the forge (README.md, section 6): a user other than the
  configured one, a `401` or `403`, or a `4xx` that asking again would
  not mend (skein's `forge-connection.md`, sections 3 and 8).

## 3. Forgejo's API as temper uses it

Held by skein's `forge.md`, section 5, and `forge-connection.md`,
sections 3 to 5.

### 3.1 Conventions

Held by skein's `forge.md`, section 5.1.

### 3.2 Each operation

Held by skein's `forge.md`, section 5.2.

### 3.3 Decoding answers

Held by skein's `forge.md`, section 5.3.

### 3.4 Failures

Held by skein's `forge.md`, section 5.4.

### 3.5 What the protocol layer keeps

Held by skein's `forge-connection.md`: each forge's start in section 3,
the repositories and the users in section 4, the API tokens in section
5.

## 4. The engine's marks

What the engine writes inside what it creates: keys' markers, and
nothing else. No record, outcome or page of the engine's is written into
the forge, and no call of skein's `forge.md`, 5.2 writes a label: labels
are for display only, and no decision reads one (`../domain/forge.md`,
section 4).

- **A creation's key,** at the head of an issue's body, a comment or a
  review, is a marker: its form, its decoding and its bounds are
  skein's (`forge.md`, section 6). temper's word is `temper`, so a
  marker reads `<!-- temper:key <the key in lowercase hexadecimal> -->`
  and a line feed, then the text. An HTML comment, which Forgejo's
  rendering hides.
- **The key is the client's** (`effect_key`, skein's `forge-client.md`,
  section 7): version 1, the namespace's length as two bytes
  big-endian, the namespace, then the effect's identity, opaque bytes
  to the client. temper's namespace is the one the root derives from the
  deployment's id; its identity is its effect's purpose and its numbers,
  which the connector's top encodes (`../domain/forge.md`, section 6).
  So the marker carries the deployment's id, and deployments that share
  a repository tell their creations apart (jig's `domain/connectors.md`,
  4.4). The key's first byte is its version; the marker has none of its
  own.
- **Decoded** as skein's `forge.md`, section 6 says; a body whose marker
  carries another word has no key. Whether to believe a key is the
  client's: its version, its namespace, and whether its author is the
  forge's writer (skein's `forge-client.md`, section 7). A person who copies a
  marker into their own comment gives it no provenance.
- **Bounded.** The key is within the client's `op_bytes`, the marker
  twice its length and 21 bytes more (the word's 6 and 15), and the body
  sent the marker and the text, which `body_bytes` holds (skein's
  `forge-connection.md`, section 7).
- **The worlds** write the same marker (the forge connector world's
  translation, `tests/engine/forge/client/src/translate.rs`); the
  protocol world (section 10) runs the component's own encoder and
  decoder.

## 5. Webhooks

Forgejo's deliveries, made hints: they cut polling's latency, and decide
nothing. Polling stands as it is with no webhook (section 6;
`../domain/forge.md`, 5.2): its `poll`, `poll_max`, `hinted` and `slow`
are unchanged, and it finds whatever a delivery lost, refused or never
sent told of. A delivery reaches the engine on loopback; a forge on
another host polls alone (setup.md, section 15).

- **A delivery's signature, its events and its hints** are skein's:
  how Forgejo signs a body, the events a hook subscribes to, the hint
  each makes, who caused it, and the bounds on a delivery (skein's
  `forge.md`, section 7).
- **The listener,** its connections, the verification as a body streams,
  the answer first, each refusal and the bounds are skein's (skein's
  `forge-connection.md`, section 6).
- **The webhooks' listener's address,** `application.webhooks`,
  `{"listener": address}` (configuration.md, section 3): a socket
  address on loopback (in `127.0.0.0/8`, or `::1`), apart from the
  clients' listener (`../client/README.md`, section 8). The routing hands it to
  the component, which takes `POST` on one path per forge,
  `/hooks/<number>`, the forge's `number`.
- **Each forge's secret,** its `webhook_secret`: a member of its
  `forges` entry naming a private file (configuration.md, section 6),
  read by the binary and handed over as a value (section 2). It is
  optional, and present only with `application.webhooks`: a
  `webhook_secret` with no `webhooks` refuses the start, naming its
  path. A forge with no secret has no path (skein's
  `forge-connection.md`, section 6).
- **What the domain does with a hint.** It drops its own writes'
  echoes: a creation's key for an item, the pusher for a push
  (`../domain/forge.md`, 5.2; skein's `forge-client.md`, section 4). A
  hint for a repository the deployment has not adopted goes up all the
  same; the domain drops it.
- **What has no delivery** is found by polling alone
  (`../domain/forge.md`, 5.2): a commit status, which Forgejo never
  delivers, so CI that a system other than Actions reports and the
  verdicts temper posts.
- **What Forgejo settles after it delivers** (a pull request's
  mergeability, a job's commit status: section 13) is the domain's to
  read again, within `settle` of the move (`../domain/forge.md`, 5.2);
  the component hints at once.
- **A listener that fails** while the engine runs stops the engine, as
  the clients' listener's does: the service drains as at a shutdown, and
  whatever starts `temper serve` again (the box, a service manager)
  starts it with a listener again.
- **Not sent again.** Forgejo does not resend a failed delivery (skein's
  `forge.md`, section 7): one lost while `temper serve` is down, refused
  for its size or its signature, or never sent is found by polling. A
  hint that arrives before the engine's domain is ready is dropped, and
  the keep's first listing finds what it told of.
- **Hooks are set up by reconcile, or by the person,** at
  `http://<listener>/hooks/<number>`, with the forge's secret, the
  events of skein's `forge.md`, section 7, JSON and active (setup.md,
  sections 4, 5 and 10; skein's `forge-setup.md`, section 4.3, step 6).
  A repository with no hook polls (`../domain/forge.md`, section 4).

## 6. The load rules, and where each is kept

A rule kept in skein's crates points to the section that keeps it. The
values are temper's: `../domain/forge.md`, 5.2 and 5.4.

| Rule | Kept by |
|---|---|
| No listing without a filter on a periodic path; no history | the client's one keep-up listing per repository, always `since` (skein's `forge-client.md`, section 4; the listing, `forge.md`, 5.2); the slow pass reads live resources one at a time (`forge-client.md`, section 4) |
| Calls per pass do not grow with history | one listing per pass, nothing listed by label (skein's `forge-client.md`, section 4); checked by temper's cost checks (10) |
| Cache by change token | comments and reviews by revision (skein's `forge.md`, 5.3); repositories' paths and users cached (`forge-connection.md`, section 4); a pull request read on a backoff (`forge-client.md`, section 4) |
| One path to CI; never a closed pull request's | statuses of the head only (skein's `forge.md`, 5.2); a closed item leaves the working set when its owner stops naming it (`forge-client.md`, section 3; temper's top, `../domain/forge.md`, 5.1) |
| API only; `page` and `limit` always; the forge's cap | skein's `forge.md`, 5.1 |
| Contents only when a decision reads them | a pull request's files and a comparison, read for a brief or an overlap, each file's reads charged (skein's `forge.md`, 5.2; `forge-client.md`, section 6) |
| Webhooks: hints, answered first, merged, targeted, echoes dropped | skein's `forge.md`, section 7 and `forge-connection.md`, section 6; the client merges hints into one earlier pass (`forge-client.md`, section 4); temper's domain drops its own echoes, by key and by pusher (`../domain/forge.md`, 5.2) |
| Closed items by deltas, not sweeps | the keep-up listing's `since`: a closed item is listed only when it changes, and its owner stops naming it (skein's `forge-client.md`, sections 3 and 4) |
| Backoff and jitter; honour the rate; no blind retries of writes | the client's backoff and budget (skein's `forge-client.md`, sections 4 and 6); `RateLimited { after }` and no retry in the component (`forge.md`, 5.4); keyed creations found again after a `Timeout` (`forge-client.md`, section 7) |
| Slow cadences; webhooks for latency, polling for liveness | the client's `poll`, `hinted` and `slow`, unchanged by webhooks (skein's `forge-client.md`, section 4); a forge with no hook, or on another host, polls alone (section 5) |

Nothing the engine writes changes often: it writes the forge only to
make an effect (`../domain/forge.md`, section 6), never as a heartbeat.

## 7. Entities, limits and the worst case

Held by skein's `forge-connection.md`, section 7. temper's values are
its `limits` member (configuration.md, section 7), set in
`temper-service`'s profile `small`.

## 8. State machines

Held by skein's `forge-connection.md`, section 8.

## 9. The fake forge's protocol layer

Held by skein's `fake-forge.md`, section 4.

## 10. Testing

The tiers, the fakes and the layout are `../domain/testing.md`'s (sections 2,
4.2 and 7). The kit's own tests are skein's. The forge's, in temper:

- **Step tests** of the documents, the encoders and decoders, the call
  plans and the marker: skein's `forge.md`, section 11.
- **The machine world** and the protocol world of the component and the
  fake forge: skein's `forge-connection.md`, section 10.
- **temper's protocol world,** `tests/engine/protocol-forge`: temper's
  forge component, which is `skein-forge-connection` handed temper's
  values, and the fake forge's protocol layer, skein's, joined by
  in-memory streams cut at random, running the forge connector world's
  stories (`tests/engine/forge`, `../domain/forge.md`, section 18)
  through bytes: a change produced, opened, checked, queued and landed;
  a creation and a merge whose answers were lost, found after a restart
  and never made twice; a failed head repaired and checked again; a held
  branch moved by another hand, reported as drift. Its invariants: what
  the engine's domain asked is what the fake's domain did; each call is
  answered once, however its streams are cut; the API token reaches no
  record, request or fact. The fake's deliveries reach the engine's
  listener signed, late, out of order or never, so its stories run with
  hints and without them.
- **The cost checks,** in the forge connector's world counting
  `Request::Forge`, and in the protocol world counting HTTP requests:
  - **idle:** over a span with no change on the forge, the calls made are
    at most the keep-up listings, one per repository every `poll`, the
    held pull requests due on their backoff, and the slow pass's reads
    every `slow`; the bound is computed from the limits;
  - **history-free:** the same scenario on a forge preloaded with ten
    times the closed items and comments makes the same calls while idle;
  - **busy:** listing pages per pass are at most the changed items over
    the page size plus one, and item reads at most the changed items
    held times one plus the pages of their new comments;
  - **honest costs:** in the protocol world, the requests the fake served
    equal the sum of the costs the answers reported, the forges' starts'
    reads aside.
- **The real Forgejo check,** its transcripts, its facts and the
  throwaway Forgejo it runs against (`../domain/testing.md`, 2.6): skein's
  `fake-forge.md`, sections 6 and 7.
- **The pin** is skein's (`forge.md`, 4.3): one patch release of Forgejo
  v17.0 and one of forgejo-runner v13.2, constants the throwaway
  Forgejo, the conformance and owned mode share (setup.md, 6.1); a
  binary of another release is refused, naming it. A later patch changes
  skein's constants and records, and not this document.
- **The process tier** runs the forge component's start, polling and
  webhooks against an owned Forgejo (`../domain/testing.md`, 2.6; setup.md,
  13.4).

## 12. What others owe

The boundary, the configuration and the setup's hooks are sections 2
and 5's. What an owner owes the component is skein's
(`forge-connection.md`, section 9). The forge client
(`skein-forge-client`, owned by temper's forge connector,
`../domain/forge.md`, section 4) owes the component:

- **`Read::Locate { owner, name }`,** answered `Answer::Located {
  repository }`, addressed with the forge's number and repository 0
  (skein's `forge.md`, 5.2); its answer's shape checked as every other's,
  a located id never 0, and its owner and name counted in the call's
  bytes.
- **Costs,** as the boundary carries them: `Event::Answered { call,
  cost, result }`, `cost` the requests the call wrote, cache fills
  included, 0 for none; the budget charges it (`../domain/forge.md`,
  5.4; skein's `forge-client.md`, section 6).
- **Hints,** as the vocabulary has them: `Hint { repository, change, key
  }`, with the pusher on `Change::BranchMoved`, so the domain drops its
  own echoes (section 5; skein's `forge.md`, section 7).
- **Shared limits:** the client's `calls`, `rows`, `answer_bytes`,
  `op_bytes` and `lifetime`, and the forge's `repositories` and
  `name_bytes`, are given to the component too, which sets and checks
  its own by them (skein's `forge-connection.md`, section 7).

## 13. What temper relies on

Held by skein's `forge.md`, section 9: what the component and the forge
connector rely on of Forgejo, on the pinned release (section 10), each
named by its fact in the conformance's `facts.txt` (skein's
`fake-forge.md`, section 7). What no fact records yet is open: the kit's
in skein's `forge.md`, sections 9 and 12, temper's own in section 14.
temper's part:

- **What it settles after it delivers.** temper relies on neither order
  of a delivery and the state it settles, only on each settling within
  `settle` of the move, 30 s in the profile `small`
  (`../domain/forge.md`, 5.2).
- **The setup's, and git's.** What the setup relies on is skein's
  (`forge-setup.md`, section 8; setup.md, 13.3); what git relies on of
  Forgejo (a renamed repository's redirect, a protected branch's refusal
  of a push) is git.md's (section 3 and 4.8), named in
  `tools/git-transcripts/facts.txt`.

## 14. Open questions

- **The kit's own** are skein's: `forge.md`, sections 9 and 12;
  `forge-connection.md`, section 11; `forge-client.md`, section 11. They
  hold every question and every unrecorded fact about the vocabulary, a
  dialect or the component, among them how long threads are read, a
  dismissed review, contents on demand, a file too large, bodies past
  `body_bytes`, an edit within a creation's second, the merge style, an
  expired or revoked token, what comments page, and a job no runner took.
- **What no fact records yet and temper's domain relies on,** read from
  Forgejo's API description and sources alone (section 13):
  - that the push workflow an update starts runs on exactly the head
    the update made, so CI is read again there (`../domain/forge.md`,
    section 10);
  - that a merge after its pull request's base was changed lands in the
    new base: the pull request is read afresh before a merge, and one
    retargeted since the decision is not merged, but nothing makes the
    merge itself conditional on its base (`../domain/forge.md`, section
    6).
- **GitHub,** later: skein's `forge.md`, section 10, and its setup,
  `forge-setup.md`, section 5.
