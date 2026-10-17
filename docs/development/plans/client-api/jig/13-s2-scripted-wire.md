# Supplement jig-13.s2: the test client wire carries the scripted family's records

Added by ruling 205 (jig 13.4's block of 2026-10-17T01:44:01Z). jig 13.4
needs its landing. Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this plan",
"Generated code" and "The run's steers"; `jig/13-reference-ui.md`, "Rules
that bind it" and 13.3.

Scope: jig's `client/domain/root.md`, 234-237: the family's records are
"in the application's wire crate, `<application>-client-protocol-wire`,
which encodes what the owner asks and decodes what the client hands
back". 13.4 sends the family sides' values "encoded by 13.3's codec",
and that codec carries the test connector's `ClientDocument` and
`ApplicationOutcome` alone: the scripted family's own documents and
answers have no carrier. This step gives them one, lossless.
Needs: jig 13.3 (landed, jig `68a2d62fa`). It runs in a slot of its own,
as 13.s1 does, beside 13.4.
Files, at jig `795ca6bc3` (follow each by its name):
- `testing/jig-client-test-protocol-wire/schema/test-client-v1.schema`:
  records for `testing/jig-client-test-domain/src/family.rs`'s `Item`
  (:37-47, with `ItemState`), `Parameters` and `PartDocument` (:48-61),
  and the answers `Touched`, `Moved`, `Gone` and
  `Refused(EntranceRefusal)` (:152-165), each field as `family.rs` holds
  it. Default: `enum FamilyDocument { item, part, connector:
  ClientDocument }` and `enum FamilyAnswer { touched, moved, gone,
  refused, connector: ApplicationOutcome }`; `ClientDocument` and
  `ApplicationOutcome` stay as they are. v1 is edited in place, as jig
  `23e3fbbdb` edited it in this run; the README's "Format versions"
  names no owner for it.
- its generated code and goldens, written only by the README's
  "Generated code" command for the test client wire;
- the codec: `testing/jig-client-test-protocol-wire/src/codec.rs`, and a
  module of its own beside `codec/{documents,facts,parts}.rs`;
- `testing/jig-client-test-domain/src/family/wire.rs`: maps
  `family::Document` and `family::Answer`, every variant, to the wire and
  back;
- each match on a changed enum that `heavy cargo check --workspace
  --all-targets --keep-going --message-format short` names (the crate's
  users: `testing/jig-test-service`, `tests/client-protocol`,
  `tests/client-wire`); list each in the commit body.
Shape: `family::wire`'s new functions return the family's value equal to
the one encoded, for each variant. `request`'s `Invalid::MissingValue`
for `Ask::Touch` without parameters (`family/wire.rs:60-61`) stays; a
story that sends one is a question before `land`.
Test shape: in `tests/client-wire/tests/test_client_wire.rs`, one round
trip per new kind (item, part with and without parameters, touched,
moved, gone, refused), its values as the scripted world's fixtures build
them (`tests/client/src/scripted.rs:32-40`), request ids and revisions
named (README, 23:45Z, point 2).
Proof: the wire layer, `tests/client-wire`; 13.4 proves the root on it
in the joint world.
Don't: no change to `jig_test_connector`'s types or to the test
connector's schema (`testing/jig-test-store/schema/test-connector-v1.schema`);
no field dropped, defaulted or inferred; no generated file edited by hand.
Size: aim 220 hand-written lines (schema about 25, codec about 100,
`family/wire.rs` about 45, tests about 50); generated code and goldens
apart. Past the aim, the report gives the counts.
