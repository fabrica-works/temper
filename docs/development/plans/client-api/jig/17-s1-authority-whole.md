# Supplement jig-17.s1: an authority crosses whole

Added by ruling 319 (the designer's answer of 2026-10-18T06:20:06Z, to
the question of 2026-10-18T05:58:00Z). Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan"; `code-organization.md`, 6; jig's `client/domain/contract.md`,
5.3 ("An authority crosses whole"), `client/domain/client.md`, 14 and
16, `client/domain/view.md`, 7.

Scope: the engine never cuts an authority's lists at `documents.lines`,
nor refuses a task document or its Amend offer for one: its bounds are
the authority's, derived, never configured; the link holds an authority
whole within a bound of its own; the view shows part of it.
Needs: nothing.

Shape:
- Engine: `DocumentLimits` (`jig-domain-core/src/documents.rs:88`)
  gains `grants`, `executors` and `implications`, derived where
  `segments` is (`jig-protocol-config/src/overrides.rs:524`;
  `profile/small.rs:158`, beside `name_segments`) and refused as
  overrides as it is (`src/tests/overrides.rs:381`): the most of
  `authority.grants` and `tasks.authority_grants`; the most of
  `authority.executors` and `tasks.executor_kinds`;
  `authority.implications`. `task_tree::authority`
  (`documents/task_tree.rs:338-373`) sizes grants and notes' scopes by
  `grants`, executors by `executors`, and cuts nothing;
  `AuthorityLine.cut` goes, and `jig-protocol-engine-clients`'
  `translate/owner.rs:262` writes the wire's `cut` false. These go:
  `act_authority`'s refusal (`documents.rs:456-459`; its callers,
  `acts/bounds.rs:76`, `:165`, `:168`, take `task_tree::authority`),
  `task_tree.rs:736-745`, `documents/offers.rs:38`'s `!authority.cut &&
  !ceiling.cut` and `:59-62`. `acts/bounds.rs:172-175` bounds
  implications by `implications`; `offers.rs`'s grants check and
  `heap_worst_case` take the new bounds.
- Link: `jig_client_domain::Limits` gains `authority` beside `rows` and
  `segments`, 8 in jig's fixtures; each authority site of
  `jig-client-protocol-link` that takes `client.rows` takes it instead
  (`git grep -n rows -- crates/jig-client-protocol-link/src
  ':!crates/jig-client-protocol-link/src/tests' | grep -E
  'grant|kinds|note_resource|scopes|implication'` prints 17 lines: the
  16 sites and `translate/asks.rs:213`'s signature). A document whose
  authority the wire marks cut is refused, too large.
- View: `jig-client-view/src/owner_authority.rs`'s `body()` shows at
  most `rows` entries of each list, then `AuthorityPhrase::Cut` once,
  its English "More are held than shown"
  (`owner_authority_words.rs:142`). `document_nodes` and
  `worst_case_words` stay.
- Tests, one each: the engine serves a task document to its owner,
  whose role holds `authority.grants` grants, at `documents.lines` 2,
  with Amend `Admission::Bounded` and the role's grants whole; the link
  decodes an authority of `authority` grants at `rows` 2 and refuses
  one more, too large; the view shows `rows` grants and the mark for a
  longer authority.
- Size: aim for at most 150 production lines; past it, the report
  says why. Don't: change the wire's schema (client-v8); change what
  `rows` bounds besides an authority's lists; raise `limits::compose`;
  change creating's charters and procedures (`acts/bounds.rs:47-63`,
  the plan's `later.md`).

Proof: land's gate (the three tests above are step tests). The
repository downstream of jig repins to this step in its own step.
