# Supplement temper-04.s1: the repin onto jig's whole authorities and busy watches, and a stream's room

Added by ruling 319 (the designer's answers of 2026-10-18T05:52:56Z and
06:20:06Z). Read first: temper's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan" and "Pins"; `temper/04-engine.md`, the corrections after the
designer's answers of 2026-10-18T05:52:56Z and 06:20:06Z; temper's
`configuration.md`, 7; `client.md`, 5.4.

Scope: temper repins to jig-17.s1 (an authority crosses whole) and
jig-17.s2 (a busy watch is opened again), and gives a client's stream
the watch room configuration.md, 7 asks, 16.
Needs: jig-17.s1's and jig-17.s2's landings (README, "Pins": its pin is
the later of the two, or temper's present pin if it holds both).

Shape:
- `crates/temper-client-shell-terminal/src/limits.rs`: `authority: 12`
  beside `segments: 9` (`:28`), the process tier's `authority.grants`
  and `tasks.authority_grants`.
- `crates/temper/tests/configuration.rs`: three assertions after the
  paths' (`:29`): the engine's `documents.grants`,
  `documents.executors` and `documents.implications` each at most
  `native.authority`.
- `tests/engine/domain/tests/forge_routes/paths.rs`:
  `a_change_on_the_forges_deepest_branch_is_served_in_its_task_document`
  first reads the producer's task document as the world's owner
  (`policy.roles[0]`), before `sign_in_reader` (`:38`) takes its
  session, and asserts it served with `Act::Amend`'s admission
  `Admission::Bounded`; the reader's read stays.
- If the start refuses memory, `engine.memory_bytes` rises by what
  the refusal names.
- **A stream's room:** a temper constant of 16 beside `VIEWS`
  (`crates/temper-engine-domain/src/limits.rs:103`), set as the
  clients component's `watches_per_stream` where its `Env` is built
  (`crates/temper-service/src/clients.rs:112`); in
  `crates/temper/tests/configuration.rs`, three assertions beside the
  documents' (`:4`): that constant, `VIEWS.watchers_per_party` and
  `VIEWS.watchers` each at least `native.watches`.
- jig-17.s2 changes no temper code: its repin repairs are what jig
  landed, as README, "Pins" says.

Proof: the world layer, once: `heavy cargo nextest run -p
temper-engine-domain-world -E 'test(a_change_on_the_forges_deepest_branch_is_served_in_its_task_document)'`
(the owner's read served), and land's gate.
Size: at most 20 production lines; the repin's repairs counted apart
(README, "Pins": at most 200).
Don't: work around a refusal or a busy `me` in temper's code; set
`documents.lines` for an authority.
