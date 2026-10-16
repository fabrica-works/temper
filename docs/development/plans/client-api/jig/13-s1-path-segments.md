# Supplement jig-13.s1: a path's segments bounded apart from the rows; the view cuts a long path

Added by ruling 143, from the design answer of 2026-10-16T15:20Z (jig
`7efaeef4d`: `client/domain/contract.md`, 5.3; `client/domain/client.md`,
14 and 16; `client/domain/view.md`, 7 and 10; `protocol/client.md`, 3).
temper 04.4 needs its landing. Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`, the sections above; this plan's README,
"Rules of this plan"; `jig/13-reference-ui.md`, "Rules that bind it".

Scope: `contract.md`, 5.3: "A document carries each whole, with as many
segments as the engine admits in a name (the most of its authority's and
its tasks' bounds on a name's segments): the engine never cuts one, nor
refuses a document for one." `client.md`, 14: "a path's segments: ... a
bound of its own, never a window's rows; the application sets it at
least the most its engine admits in a name". `view.md`, 7: "When the
phrase before them and their segments would pass a node's text, the view
leaves out the first segments, shows in their place the mark `…` ...
then the last segments that fit, the last one always. ... what the view
acts on (a filter by a resource, a draft's pattern) takes the whole
path". jig-view's compose is not raised (`view.md`, 10).
Needs: jig 13.2's landing (the step follows 12.1's and 13.2's moves of
the files below).
Files, at jig `e1b483760`'s code (each line moves with 12.1 and 13.2;
follow it by its name):
- the client's limits: `segments: u32` beside `rows` and `text`
  (`jig_client_domain::Limits` after jig 12.1), set by the application in
  code; each literal of it gains `segments`, 4 in jig's (the small
  profile's `authority.segments`, `tasks.hold_segments` and
  `tasks.authority_segments`, `profile/small.rs:46, :66, :117`): at
  `e1b483760`, `git grep -n 'conversation_words:' -- '*.rs'` prints 10
  literals of the page's limits, of which 12.1 makes the client's; list
  each one in the commit body;
- link: `crates/jig-client-protocol-link/src/limits.rs:31, :41, :79`
  (`part_resource_path`, `owner_part_key_path_value`,
  `owner_pattern_segments`, each `client.rows` today),
  `translate/documents.rs:312, :330`, `translate/pages.rs:84, :108`,
  `translate/pages_part7.rs:107, :117` take `segments`;
- view (`crates/jig-client-view/src/`): `owner_chip.rs:39-45, :95-97`,
  `jig-client-domain`'s `contract/resource_names.rs:9-11`
  (`resource_path_fits(path, rows, text)`), `owner_amendment.rs:27-29`,
  `owner_historical.rs:571-573`, `sections/project/owner.rs:203-205`
  take `segments`; the cut where paths are written (`words/held.rs:118-122`,
  `owner_authority.rs:117-125`, `policy_edit.rs:107-108`);
  `limits.rs:77-80`'s node text holds a phrase, the mark and
  `literal_segment::max_bytes(text)`;
- core: `crates/jig-domain-core/src/documents/task_tree.rs`, `path`
  (`:328-340`, `documents.lines` today) and the approval part
  (`:404-411`), and `project_note.rs`' `path` calls (`:147-148`), by the
  names' bound: the most of the three above, derived where the engine's
  limits are made; `documents.lines` bounds short lists alone.
Test shape:
- a task holding a name of `authority.segments` segments, at
  `documents.lines` 2, has its task line and `task` document served;
- the link decodes a path of `segments` segments at `rows: 2`, and
  refuses one more as too large;
- a path past a node's text shows `…` and its last segments, and the
  notes filter made from it carries the whole path.
Proof: each test beside its kind (the engine's task line in
`tests/core/tests/task_queries.rs`, its `task` document in
`tests/core/tests/application_documents.rs`; the link's decode in
`crates/jig-client-protocol-link/src/tests/documents.rs`; the view's cut
in `crates/jig-client-view/src/tests/owner_chip.rs`); jig's gate, by
`land`.
Size: aim for at most 160 production lines; past it, report why.
Don't: cut a path in the client's data or the engine's documents; change
what `rows` bounds besides paths; raise `compose`; give the engine a
configured limit for a path; name temper (`git -C <worktree> grep -n -i
-w temper -- crates testing tests examples scripts` prints nothing).
Done when: `git -C ~/src/rust/jig log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: jig-13.s1$' --format=%H -1`
prints its landing.
