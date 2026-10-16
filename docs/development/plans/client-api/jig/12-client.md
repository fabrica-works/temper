# Session 12: `jig-client`, the client the application holds

This session makes `jig_client::Client` (`crates/jig-client`): jig 11's
`jig-client-domain` and jig 10's link composed into one step machine,
with client.md, 4's surface (`step`, `fire`, `next_deadline`;
`document`, `pending`, `link`). It gives the client the link's five
states, judged from the stream and its own watch of `me` (12.1); the
small store's barrier and kept work on its surface (12.2); `SignIn {
with }` and `SignOut` (12.3); versions per owner, with families as bytes
under their owner (12.4); and a typed transport for the client's world,
whose engine is scripted in the contract's types (12.5). It is the
meeting point (README, Rules): its first increment runs a thin story of
the new client against the real engine in `tests/client-protocol`, and
each increment after it grows that story; from here each client
increment proves against the real engine.

## Read first

- jig's `docs/design/client/domain/client.md`, sections 1 to 4, 6, 8
  (8.1 and 8.2), 11, 12, 14 and 16.
- jig's `docs/design/client/domain/root.md`, sections 3, 5, 7 and 15.
- jig's `docs/design/client/domain/testing.md`, sections 2, 4, 5 and 6.1.
- jig's `docs/design/protocol/client.md`, sections 2, 4, 8, 9, 10 and 11.
- jig's `docs/design/README.md`, section 10.
- The plan's `findings/wire-and-client.md`, sections 5 (c, e, j, k) and
  6, and `code-organization.md`, sections 1, 3, 5, 6 and 7.
- Sessions 10 and 11's files: their sketches are this session's types.
- skein's `programming-model.md`, sections 3, 4.4, 4.5, 6.3 and 9;
  `testing-strategy.md`, sections 2.5, 6 and 7.

## What the design asks

- "`jig-client` composes the two into `jig_client::Client`, one step
  machine" (client.md, 3); its surface is client.md, 4's sketch: `step(now,
  Event, &mut Queue<Request>), fire(now, &mut Queue<Request>),
  next_deadline()`, `client.document(handle), client.pending(key),
  client.link()`.
- The link's states (8.2): "signed out: no sign-in is live; starting:
  signed in, and `me`'s first delivery has not come; live: the stream is
  live, and `me` has been delivered on it; offline: the stream dropped,
  or went silent past twice its heartbeat ...; out of date: the engine
  serves no version of jig's that the client speaks. It sends nothing,
  and keeps every request." "Beside its state, the link names the party
  confirmed, each owner whose version the engine does not serve, and a
  sign-out that did not reach the engine."
- The small store (11): "A request's record leaves in a `Save`, and the
  client gives the request's first `Exchange` only once the transport
  says `Stored`. `NotStored` ... retires the requests the write carried,
  unsent, each answered *not kept*."
- `SignIn { with }` (12): `Link(secret)`, `Service(credential)`,
  `Held(credential)`, `Transport`; signed out "drops the credential
  held, ends every handle (`Ended`, signed out), parks every request,
  and tells `Link` signed out"; "A sign-out that does not reach the
  engine ... leaves the party signed in"; "A sign-out's answer is its
  sign-in's".
- Versions per owner (8.2; protocol/client.md, 9): "Outside a range of a
  family's owner, that owner's requests stay kept and unsent, its reads
  are answered and its watches end as version not served; outside
  jig's, the link is out of date."
- Families as bytes (client.md, 4; root.md, 7): "The client never reads
  a family's bytes. It reads their owner, their version and their size".
- The client's world: "the engine, scripted; the transport, played by
  the world" (testing.md, 2 and 4).

## Choices this plan makes

- **The domain and the link keep their vocabularies;** `jig-client`
  maps the domain's `Down` onto the link's events and the link's
  requests onto the domain's `Up` (jig 11's choice). The map is
  exhaustive both ways: a variant added on either side breaks the build
  (development.md, 5.3).
- **The link's limits derive from the client's.** The link's
  `Limits.client` becomes `jig_client_domain::Limits`, which gains the
  decoding bounds the link reads today from the page's limits (`rows`,
  `text`, `projects`, `charters`, `fields`, `heartbeat`); its users build
  them from the same values.
- **The small store's barrier is the client's, for keyed requests
  alone** (findings/wire-and-client.md, 5j): reads and watches leave at
  once; a step's records go in one `Save` or one `Erase`; `NotStored`
  answers each keyed request the write carried `NotKept`.
- **The thin story** lives in `tests/client-protocol` (findings 6:
  `src/engine.rs` runs the real core under the clients' component over
  HTTP bytes): a scripted application in front of `jig_client::Client`,
  over the native shell's `Connections` and the world's `Net`. Its
  request is `Stop` of task 9 in project 1, which does not exist: the
  engine refuses it, durably, and no shape of work is named (findings
  6), so jig 14 does not rewrite it.
- **`me` is still `Subject::Frame`** (jig 14.1 and 22.1 reshape it).
- **`jig-client` re-exports `jig-client-domain`'s types,** the client's
  surface (client.md, 3), so an application, and the reference UI, depend
  on `jig-client` alone (view.md, 2); code-organization.md, 2's "no
  re-export" forbids only aliases kept for old paths.
- **The typed transport** (`Client::typed`) carries the domain's `Down`
  and `Up` instead of the link's bytes, for the client's world, whose
  scripted engine speaks the contract's types (testing.md, 4); no shell
  uses it (Questions).

## Rules that bind it

- **The meeting point** (README, Rules): each increment's Proof runs
  against the real engine in `tests/client-protocol`, never only against
  `jig-client-scripted-engine`; the story grows with each increment.
- **No HTTP and no page in `jig-client`** (code-organization.md, 6):
  12.1 adds the boundary test
  `client_crate_links_only_its_domain_link_and_foundation` (below).
- **The credential is never in the small store** (client.md, 12): the
  link's `Kept` records hold none; 12.3's test
  `a_held_credential_never_reaches_the_small_store` checks it.

## Increments

### 12.1 `jig_client::Client`: the domain and the link composed; the link's states; the story begins
Scope: the new crate, its surface (client.md, 4; sketch), the
composition, and the link's states signed out, starting, live, offline
and out of date (8.2), with `Told::Link` and `link()`; the client's own
`me` watch opened when a sign-in is live (moved from jig 11.2's
`Start`), its party and retention given to the table; the thin story's
first part against the real engine: signed in by the printed link, `me`
live, the project `me` names read.
Needs: jig 10's link vocabulary and jig 11's domain on `main` (the
session's start).
Files: new `crates/jig-client/{Cargo.toml, src/lib.rs, src/client.rs,
src/limits.rs, src/tests.rs, src/tests/fixtures.rs}`; workspace
`Cargo.toml` and `Cargo.lock` (shared);
`crates/jig-client-domain/src/{boundary.rs, domain.rs, link.rs (new),
watches.rs, limits.rs}`; `crates/jig-client-protocol-link/src/limits.rs`
(`Limits.client`) and its users: `tests/client-protocol/src/fixtures.rs:102`,
the link's `src/tests/fixtures.rs:37`, and downstream its terminal's
`limits.rs`; `crates/jig-client-protocol-link/src/{boundary.rs, store.rs}`
(the small store's names, below) and their users: the native shell's
`shell.rs:111-112, 246` and `files.rs` (`loaded`),
`tests/client-protocol/src/{front.rs, front/persistence.rs:38,
native.rs}`, the native shell's `src/tests.rs:257, 278`; downstream, its
terminal's `client.rs:328`, `route.rs:52-53` and `tests.rs:53-82`; `tests/boundary/src/tests/graph.rs` (`client_step` gains
`jig-client`; the new test below) (shared);
`tests/client-protocol/{Cargo.toml, src/lib.rs, src/client.rs (new),
tests/client.rs (new)}`.
Shape: `jig_client::{Client, Event, Request, LinkView, Limits, Unusable,
max_out, worst_case}` as the sketch (`Event::Up` and
`Request::Down` come with 12.5); `jig_client` re-exports the client's
surface it names: `jig_client_domain`'s `contract`, `Asker`, `Handle`,
`Key`, `Held`, `Ask`, `Family`, `Readable`, `Watchable`, `Readout`,
`Answered`, `Outcome`, `Pending`, `Progress`, `State`, `Latest`,
`Document`, `Why`, `Told`, `Link`, `Down`, `Up`, and the link's `With`,
`Message`, `Streamed` and `Record`; `jig_client_domain::Link { SignedOut,
Starting, Live, Offline, OutOfDate }`, `Told::Link { link }`,
`Up::{SignedIn, Served { served: bool }}`. Starting on the link's
`Signing::SignedIn` or a `Held` credential; live at `me`'s delivery;
offline when `me`'s watch drops (the link drops the stream when it ends
or is silent past twice its heartbeat, protocol/client.md, 4); out of
date at a hello whose jig range the link judges not served (jig 11.1's
`serves`), after which nothing is sent and the table stays parked. The
table is up while live, confirmed to `me`'s party, with `me`'s
retention. The link's small-store names become 4.3's: `Write { key,
record }`, `Remove { key }`, `Written`, `WriteFailed` and `StoreRead {
key, record }` are `Save { record: Record }`, `Erase { key }`, `Stored`,
`NotStored` and `Record { key, bytes }` (one type, both ways); the
client gathers a step's `Save`s and `Erase`s into one `Request::Save {
records }` and one `Request::Erase { records }`. The barrier is the
link's, unchanged, until 12.2.
Test shape: unit tests in `crates/jig-client/src/tests.rs` on
`src/tests/fixtures.rs` (made here; limits from the client world's
profile, as jig 11's fixtures take them: `jig_client_world::test::limits()`,
tests/client/src/test.rs:18), for the composition: `step` routes each
`Event` once and each request once; reference test:
```rust
#[test]
fn a_watch_asked_of_the_client_leaves_as_one_opening_on_the_stream() {
    let mut c = fixtures::Client::signed_in(PARTY);
    c.step(Event::Watch { handle: TASK, subject: task(9) });
    assert!(matches!(c.requests().as_slice(), [Request::OpenStream { credential: Some(_) }]));
}
```
The story: `tests/client-protocol/src/client.rs`, `pub struct Story`
holding a `jig_client::Client`, a `jig_client_shell_native::Connections`
(fixtures.rs:94's limits), a `jig_client_protocol_http::Mapping` (`/jig`)
and `Stream`, the world's `Engine` (src/engine.rs:27, `Engine::bounded`
with `fixtures::owner_limits()`, fixtures.rs:17) and `Net` (src/net.rs:52),
the told requests in order, and the small store as the host keeps it;
its loop follows `Protocol::advance` (src/front.rs) with one client.
Methods: `Story::new(seed)`, `sign_in()` (the engine's printed link, read
with `jig_client_protocol_http::printed`), `watch(handle, subject)`,
`read(handle, read)`, `send(key, request)`, `run_until(done, within) ->
bool`, `told_links() -> Vec<Link>`, `answered(key) -> Vec<Outcome>`,
`made(party) -> usize` (the engine's `Observed::Made` for that party),
`reload()` (12.2), `cut_answer()` (`Net::cut_answer`, net.rs:92).
Reference test, `tests/client-protocol/tests/client.rs`:
```rust
#[test]
fn a_client_signed_in_by_the_printed_link_sees_me_live() {
    let mut story = Story::new(fixtures::SEED);
    story.sign_in();
    assert!(story.run_until(|s| s.client.link().state == Link::Live, Duration::from_secs(1)));
    assert_eq!(story.told_links(), vec![Link::Starting, Link::Live]);
    assert_eq!(story.client.link().party, Some(fixtures::FIRST_OWNER));
}
```
Tests: `a_client_signed_in_by_the_printed_link_sees_me_live`,
`a_client_reads_the_project_me_names_by_its_handle`,
`a_dropped_stream_makes_each_handle_behind_then_live_with_me` (story:
the world's `Net::cut_stream`, net.rs:104; each handle's state behind
and the link offline, then each live once `me` returns),
`each_delivery_the_client_took_reaches_the_core_as_taken` (testing.md,
6.1, "A delivery taken is told": at quiescence the engine's core holds
as taken each delivery the client took, whatever ended its watch); unit:
`a_watch_asked_of_the_client_leaves_as_one_opening_on_the_stream`,
`a_reply_reaches_the_domain_as_its_decoded_answer`,
`an_application_watch_on_me_shares_the_client_s_own`,
`an_engine_not_serving_jig_s_version_leaves_the_link_out_of_date`,
`a_request_parked_offline_is_sent_once_me_returns` (client.md, 8.2),
`a_read_whose_exchange_is_lost_is_answered_unreachable` (client.md, 7).
The story keeps a referee (`tests/client-protocol/src/client.rs`'s
`Referee`), checked at each step, which this increment starts with one
rule of testing.md, 6.1: a document told `Changed` for a handle never
carries a lower revision than one told before for its subject; its
referee test `a_document_older_than_one_told_is_caught` feeds it two
documents out of order. Each story under 0.5 s, each unit test under
20 ms.
Proof: joint layer, the real engine over HTTP bytes: `heavy cargo
nextest run -p jig-client-protocol-world -E
'test(a_client_signed_in_by_the_printed_link_sees_me_live)'`, once; it
passes when the client's link goes starting then live and names party 1
(under 0.5 s). It passes; nothing else is wanted.
Don't: no HTTP, no page, no family's bytes read; no change to the
reference UI or the roots (jig 13 and jig 21).
Size: at most 300 lines (the crate about 180, the link's states about
80, the limits about 40).
Removes: nothing.

The boundary test 12.1 adds, in `tests/boundary/src/tests/graph.rs`:
```rust
#[test]
fn client_crate_links_only_its_domain_link_and_foundation() {
    let clients: Vec<_> = manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-client")).collect();
    assert_eq!(clients.len(), 1, "one client");
    let mut dependencies = clients[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(dependencies, ["jig-client-domain", "jig-client-protocol-link", "skein-lib"]);
}
```

### 12.2 Keyed requests end to end: kept before sent, answered once durable, kept across a reload
Scope: the client's barrier (client.md, 11): a step's records in one
`Save`; a keyed request's first `Exchange` only after `Stored`; reads
and watches never wait; `NotStored` answers each keyed request its write
carried `Outcome::NotKept`; `Start { kept }` restores the table's
records, each told `Kept { key, request }`. The story grows: a `Stop`
answered once after its commit is durable, and kept across a reload.
Needs: 12.1's client.
Files: `crates/jig-client/src/{client.rs, store.rs (new)}`;
`crates/jig-client-domain/src/{boundary.rs, requests.rs}` (`NotKept`);
`crates/jig-client-protocol-link/src/{boundary.rs, store.rs}` (the
link's barrier holds `Message::Request` exchanges alone, findings 5j;
`Request::NotKept { attempt }` for each held keyed attempt on
`NotStored`); `tests/client-protocol/src/front.rs` (the old roots' map
of `NotKept`); `tests/client-protocol/src/client.rs` (the story's
`reload()`, `restart_engine(offline)` as front/faults.rs:35-42 restarts
it, `refuse_next_save()`, a second client of the same party, and the
referee's rules below).
Shape: `jig_client_domain::Outcome::NotKept`; the link's
`Request::NotKept { attempt }`; the link's `Request::Answer` gains `why:
Option<Retry>` (jig 11.3's), telling busy from not ready, and a final
answer that does not decode is `Lost` with `Retry::Undecodable`, beside
its fact (protocol/client.md, 5: "lost, and reported through the
request's progress as retrying because its answer did not decode"), so
the application is told `Pending` and reads `Retrying { why:
Undecodable }`; the link's `Request::Dropped` reaches
the application as `Told::Dropped` (jig 11.3). The old
roots' path, through the link outside them until jig 13.4 (the test
root) and jig 21.2 (`ops`'s), maps the
link's `NotKept { attempt }` to its table's final answer with none (its
`Fact::InvalidAnswer`), as the link's `WriteFailed` has no other user.
Test shape: unit tests in `crates/jig-client/src/tests/store.rs`
(fixtures from 12.1); story tests in `tests/client-protocol/tests/client.rs`.
Tests: story `a_stop_is_answered_once_after_its_commit_is_durable`
(written out below), `a_stop_kept_across_a_reload_is_answered_once`
(the answer cut with `cut_answer`, the client reloaded from the story's
small store before the retry: `Kept` told for its key, the engine's
`made(1)` 1 at the end),
`a_stop_whose_record_is_refused_is_answered_not_kept_and_never_made`
(testing.md, 7: the story's store refuses the save),
`the_engine_restarting_under_a_pending_stop_answers_it_once`,
`two_clients_stopping_one_task_are_answered_once_each` (a fake party of
the engine's world, party 1, starts a chat, as stories.rs:59's
`pressure()` scripts a party, and two clients of party 1 each send
`Stop` of its task under their own keys: each key is made once, and the
second is answered as the first decided, contract.md, 4); unit
`a_keyed_request_waits_for_its_record_to_be_stored`,
`a_read_leaves_while_a_record_is_being_stored`,
`a_record_not_stored_answers_its_request_not_kept`,
`a_step_s_records_leave_in_one_save`,
`a_request_lost_while_live_is_sent_again_and_the_link_stays_live`
(client.md, 8.2), `a_kept_record_of_another_version_is_dropped_and_told`
(client.md, 11),
`a_final_answer_that_does_not_decode_is_told_retrying_as_undecodable`. The story's referee gains client.md, 17's rules: no
keyed exchange leaves before its record is stored, none for a party
`me` has not confirmed, and no queue past its limit; its referee test
`a_keyed_exchange_before_its_store_is_caught`. Reference story:
```rust
#[test]
fn a_stop_is_answered_once_after_its_commit_is_durable() {
    let mut story = Story::live(fixtures::SEED);
    story.send(STOP, Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 })));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), Duration::from_secs(1)));
    assert!(matches!(story.answered(STOP).as_slice(), [Outcome::Answered(Answered::Jig(contract::Answer::Refused(_)))]));
    assert_eq!(story.made(fixtures::FIRST_OWNER), 1);
    assert!(story.told_at(STOP) >= story.durable_at(fixtures::FIRST_OWNER));
}
```
(`Story::live` is `new`, `sign_in` and `run_until` live; `STOP` is
`Key { asker: Asker(1), number: 1 }`; `told_at(key)` is the time the
story took `Answered` for the key, and `durable_at(party)` the time of
the `Observed::Durable` whose commit is that party's `Made`.) Each story
under 0.5 s, each unit test under 20 ms.
Proof: joint layer: `heavy cargo nextest run -p jig-client-protocol-world
-E 'test(a_stop_kept_across_a_reload_is_answered_once)'`, once; it
passes when the reloaded client is told `Kept` for its key and the
engine makes the `Stop` once (under 0.5 s). It passes; nothing else is
wanted.
Don't: no answer kept in the small store; no request sent before its
`Stored`.
Size: at most 215 lines.
Removes: the link's barrier on reads and watches.

### 12.3 `SignIn { with }` and `SignOut`
Scope: `With::{Link, Service, Held, Transport}` (client.md, 12); a
service signs in with its credential as the message's credential and
holds the sign-in's credential the answer gives; under `Transport` the
client holds none and exchanges carry none; signed out (an answer's,
a read's or a watch's end, the stream ending signed out, or `SignOut`
answered) drops the credential, ends each handle `Ended { why:
SignedOut }`, parks each request, abandons each read, and tells
`Link(SignedOut)`; a sign-out that does not reach the engine keeps the
credential and the handles, `link().sign_out_lost` true; a sign-out's
answer is its sign-in's (an earlier sign-in's is dropped once a newer is
live); `me` opens on each sign-in. The story grows: signed out with a
`Stop` parked, signed in again, the `Stop` sent once.
Needs: 12.2's story.
Files: `crates/jig-client-protocol-link/src/{boundary.rs, sign_in.rs}`
(`With::{Service, Transport}`, `Message::SignInService`, a sign-in's
number); `crates/jig-client-protocol-http/src/call.rs`
(`SignInService` to `<prefix>/v1/sign-in/deployment`);
`crates/jig-client-domain/src/{boundary.rs, link.rs, domain.rs}`
(`Up::SignedOut`, the ends and parking); `crates/jig-client/src/client.rs`;
`tests/client-protocol/src/client.rs`, `tests/client.rs`.
Shape: `With` gains `Service(Secret)` and `Transport`; `Message` gains
`SignInService`; `Up::SignedOut`; `LinkView.sign_out_lost`. A service's
sign-in answered not ready is sent again after a backoff drawn from the
seed ("Not ready is *again*: a service's client signs in again after
its backoff", protocol/client.md, 8); a person's sign-out answered not
ready shows as one that did not reach the engine (8).
Test shape: unit tests in `crates/jig-client/src/tests/sign_in.rs`;
story in `tests/client-protocol/tests/client.rs`. Tests: story
`a_client_signed_out_with_a_stop_parked_sends_it_once_signed_in_again`
(the engine ends the sign-in as the protocol world's
`FrontFault::SignedOut` does, src/front/faults.rs:13; the client signs
in again by a fresh printed link; `made(1)` is 1); unit
`signed_out_ends_each_handle_once_and_parks_each_request`,
`a_lost_sign_out_keeps_the_credential_and_says_so`,
`an_earlier_sign_in_s_sign_out_answer_is_dropped`,
`a_service_signs_in_with_its_credential_as_the_message_s`,
`under_transport_exchanges_carry_no_credential`,
`a_held_credential_never_reaches_the_small_store`,
`signing_out_again_after_a_lost_sign_out_is_answered_signed_out`,
`a_sign_out_answered_not_ready_did_not_reach_the_engine`,
`a_service_s_sign_in_answered_not_ready_is_sent_again_after_its_backoff`.
Each story under 0.5 s, each unit test under 20 ms.
Proof: joint layer: `heavy cargo nextest run -p jig-client-protocol-world
-E 'test(a_client_signed_out_with_a_stop_parked_sends_it_once_signed_in_again)'`,
once; it passes when the parked `Stop` is made once after the second
sign-in names the same party (under 0.5 s). It passes; nothing else is
wanted.
Don't: no sign-in flow beyond jig's two (client.md, 12: the
application's own end in `Held`); no `Mint` (jig 17).
Size: at most 280 lines.
Removes: nothing.

### 12.4 Versions per owner, and families as bytes under their owner
Scope: the hello's ranges per owner (protocol/client.md, 9): outside
jig's, out of date (12.1); outside a family owner's, its requests stay
kept and unsent (each attempt answered `Again` by the client without
leaving, its progress `NotServed`), its reads answered at once and its
watches ended `Why::NotServed`, and `link().not_served` lists it; a
hello in range again sends what was kept, under its keys. Families ride
the same table, watches and reads as bytes under their owner (client.md,
4 and 6); the story grows: a family's request answered as its owner's
bytes.
Needs: 12.3's client.
Files: `crates/jig-client-domain/src/{boundary.rs, requests.rs,
watches.rs, reads.rs}`; `crates/jig-client/src/client.rs` (the link's
`Hello { family: Some(_), .. }` to `Up::FamilyServed`);
`tests/client-protocol/src/client.rs`, `tests/client.rs` (the family's
request with `src/family.rs:15`'s `request` and `:32`'s `answer`).
Shape: `Up::FamilyServed { family: u16, versions: VersionRange }`;
`Why::NotServed`; `Progress::NotServed`; `LinkView.not_served`.
Test shape: unit tests in `crates/jig-client-domain/src/tests/versions.rs`;
story in `tests/client-protocol/tests/client.rs`. Tests: story
`a_family_request_is_answered_as_its_owner_s_bytes`,
`an_engine_outside_jig_s_version_leaves_the_client_out_of_date_sending_nothing`
(the protocol world's `FrontFault::Versions` serving, front/faults.rs:44);
unit `a_family_owner_not_served_keeps_its_requests_unsent`,
`a_family_owner_not_served_ends_its_watches_not_served`,
`a_family_owner_served_again_sends_what_was_kept`,
`a_family_document_past_the_bytes_limit_ends_its_handle`. The fuzzy
suite gains `tests/client-protocol/tests/fuzzy_client.rs`'s
`random_scripted_applications_keep_the_client_s_rules`: a scripted
application drawing its events (watches, reads, sends, sign-outs and
sign-ins, reloads) from the seed against the real engine, the story's
referee in the loop (testing.md, 4: "A random one, for the fuzzy suite,
draws its events from the seed"), about 5 s over its seeds. Each story
under 0.5 s, each unit test under 20 ms.
Proof: joint layer: `heavy cargo nextest run -p jig-client-protocol-world
-E 'test(a_family_request_is_answered_as_its_owner_s_bytes)'`, once; it
passes when the test connector's outcome comes back as the family's
bytes (under 0.5 s). It passes; nothing else is wanted.
Don't: no family's bytes decoded in the client (the story decodes them
with the test application's wire); no typed version refusal per message
(jig 14's).
Size: at most 150 lines.
Removes: nothing.

### 12.5 A typed transport for the client's world
Scope: `Client::typed(limits, seed)`: the domain with no link, its
`Down` given out as `Request::Down` and its `Up` taken as `Event::Up`,
for a transport that carries them to an engine scripted in the
contract's types (testing.md, 4; the client's world's scripted engine).
Sign-in under it is `With::Transport`; the small store's records are the
domain's (`RequestRecord<Held>`), kept by the world.
Needs: 12.4's client.
Files: `crates/jig-client/src/{client.rs, lib.rs}`,
`src/tests/typed.rs`; `tests/client/src/joint/front.rs` (its front
carries a typed client's `Down` and `Up` through its translation to the
real core, `tests/client/src/joint/transport.rs`), `tests/client/tests/`
(the joint story below).
Shape: `Client::typed`, `Event::Up { up }`, `Request::Down { down }`.
Under `typed`, `Event::{Reply, Lost, Streamed, StreamGone}` are
dropped, and under `new`, `Event::Up` is; each is a fact the client's
host made a mistake of, and `debug_assert!`s it.
Test shape: `crates/jig-client/src/tests/typed.rs`: tests
`a_typed_client_gives_its_watch_as_the_domain_s_opening`,
`a_typed_client_takes_an_answer_as_the_domain_s`,
`a_typed_client_s_records_are_the_table_s`. Each under 20 ms. The joint
story `a_typed_client_sees_me_live_against_the_real_core` (tests/client,
`jig-client-world`, the joint world: a scripted application in front of
`Client::typed`, its `Down` and `Up` carried by the joint front to the
real core), under 0.5 s.
Proof: joint layer, the real core: `heavy cargo nextest run -p
jig-client-world -E 'test(a_typed_client_sees_me_live_against_the_real_core)'`,
once; it passes when the typed client's link goes live with party 1;
nothing else is wanted. jig 13.4's test root then runs the client's
world on it (`signing_in_and_starting_a_chat`, tests/client).
Don't: no typed path in any shell; no `Down` or `Up` under `new`.
Size: at most 100 lines.
Removes: nothing.

## Interface sketches

Checked at the base with `heavy cargo clippy -p jig-w6-sketch
--all-targets -- -D warnings` in a scratch crate (session 10's link
types and session 11's domain types stubbed beside it), passing. The
client owns the domain, the link and their map; the application owns
handles' and keys' numbers and the host's store; the transport owns
deadlines.

```rust
// crates/jig-client/src/lib.rs. The client's surface, re-exported (client.md, 3):
// pub use jig_client_domain::{contract, Answered, Ask, Asker, Document, Down, Family, Handle,
//     Held, Key, Latest, Link, Outcome, Pending, Progress, Readable, Readout, State, Told, Up,
//     Watchable, Why};
// pub use jig_client_protocol_link::{Message, Record, Streamed, With};
// `Record` is the link's (12.1: `StoreRead` renamed):
// pub struct Record { pub key: Box<[u8]>, pub bytes: Box<[u8]> }, the keeper-tagged
// storage key and the `Kept` record's bytes, as the host keeps them.
/// What the client hears (client.md, 4.1).
pub enum Event {
    /// The small store's records, as the host loaded them; before any other event.
    Start { kept: List<Record> },
    SignIn { with: With },
    SignOut,
    Watch { handle: Handle, subject: Watchable },
    Unwatch { handle: Handle },
    Read { handle: Handle, read: Readable },
    Send { key: Key, request: Ask },
    /// The transport's (protocol/client.md, 4.3).
    Reply { exchange: Token, message: Box<[u8]> },
    Lost { exchange: Token },
    Streamed { message: Streamed },
    StreamGone,
    /// The last `Save` or `Erase` written, or not.
    Stored,
    NotStored,
    /// From a typed transport only (12.5).
    Up { up: Up },
}
/// What the client asks (client.md, 4.2).
pub enum Request {
    Exchange { exchange: Token, credential: Option<Secret>, message: Message },
    OpenStream { credential: Option<Secret> },
    CloseStream,
    /// A step's records, written before the keyed requests they keep are sent.
    Save { records: List<Record> },
    /// A step's records' keys.
    Erase { records: List<Box<[u8]>> },
    /// For a typed transport only (12.5).
    Down { down: Down },
    /// For the application, routed by the asker it names; `Link` to each child.
    Told { told: Told },
}
/// What `link()` lends (client.md, 8.2 and 12).
pub struct LinkView<'a> {
    pub state: Link,
    pub party: Option<u64>,
    pub not_served: &'a [u16],
    pub sign_out_lost: bool,
    pub credential: Option<&'a Secret>,
}
pub enum Unusable { Limits }
pub struct Limits { pub domain: jig_client_domain::Limits, pub exchanges: u32 }
pub struct Client { /* domain, link (none under `typed`), the map's state */ }
impl Client {
    pub fn new(limits: &Limits, seed: u64) -> Result<Client, Unusable>;
    pub fn typed(limits: &Limits, seed: u64) -> Result<Client, Unusable>;
    pub fn step(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<Request>);
    pub fn fire(&mut self, now: Time, wall: Wall, out: &mut Queue<Request>);
    pub fn next_deadline(&self) -> Option<Time>;
    pub fn reclaim(&mut self);
    pub fn document(&self, handle: Handle) -> Option<Document<'_>>;
    pub fn text(&self, handle: Handle) -> Option<&RunText>;
    pub fn pending(&self, key: Key) -> Option<Pending>;
    pub fn link(&self) -> LinkView<'_>;
}
pub fn max_out(limits: &Limits) -> u32;
pub fn worst_case(limits: &Limits) -> Option<u64>;

// crates/jig-client-domain/src/boundary.rs, what this session adds
pub enum Link { SignedOut, Starting, Live, Offline, OutOfDate }   // Told::Link { link } (12.1)
// Up gains: SignedIn, Served { served: bool } (12.1); SignedOut (12.3);
//           FamilyServed { family: u16, versions: VersionRange } (12.4)
// Outcome gains NotKept (12.2); Why and Progress gain NotServed (12.4)

// crates/jig-client-protocol-link/src/boundary.rs, what this session adds
// With gains Service(Secret) and Transport (12.3); Message gains SignInService (12.3);
// Write, Remove, Written, WriteFailed and StoreRead become Save { record }, Erase { key },
// Stored, NotStored and Record { key, bytes } (12.1); Request gains NotKept { attempt } (12.2).
```

The map in `jig-client`, written once (an algorithm easy to get wrong):
```rust
// One step: application events to the domain; transport events to the link;
// then, until both are quiet within the step's bound:
//   the link's requests: Exchange/OpenStream/CloseStream/Save/Erase -> out (keyed
//   exchanges held until Stored); Answer/Watch/FamilyDelivery/Ended/Read/Hello/Kept
//   -> domain as Up; Signing -> domain as SignedIn/SignedOut;
//   the domain's requests: Down -> link events; Told -> out.
// The bound is max_out: each pass consumes what the other produced, so the loop
// runs at most twice per message the step admitted.
```

## Questions to raise when reached

- **The client's world and the typed transport** (testing.md, 4): the
  design has the world play the transport to a scripted engine, which
  speaks the contract's types, while the client gives the transport
  bytes. The plan adds `Client::typed` (12.5); if the designer wants the
  scripted engine to read and write wire records instead, 12.5 goes and
  jig 13.4's and jig 21.2's client worlds change.

## Done when

- `heavy cargo nextest run -p jig-client-protocol-world -E
  'binary(client)'` passes the fourteen tests this session names for that binary, in a worktree at
  `origin/main`.
- `heavy cargo nextest run -p jig-boundary -E
  'test(client_crate_links_only_its_domain_link_and_foundation)'`
  passes one test.
