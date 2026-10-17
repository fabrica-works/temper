# Supplement jig-14.s1: words release nothing; the words' release leaves the engine

Added by ruling 210, from the design answer of 2026-10-17T02:41Z (jig
`5a9a97485`: `domain/people.md`, 5.1; `domain/tasks.md`, 7;
`client/domain/contract.md`, 3.4). Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this plan"
and "The run's steers"; `jig/14-wire.md`, "Rules that bind it" and 14.3.

Scope: people.md, 5.1: "Words release nothing: a `Message` adds words to
the inbox, and words to a task the party stopped come with a `Release` in
one compound request (below), naming the hold's revision as it would
alone."
Needs: jig 14.3 (its `Message` translates to `release: false`). It runs
in a slot of its own, beside 14.4.
Files, Shape, Tests, Proof, Size and Don't: ruling 210's Decision, point
4, as written (Files at jig `5a9a97485`; follow each by its name). Size:
at most 150 lines of production code.
temper: ruling 210's point 5 names temper's sites; the temper session
that repins jig past this step edits them.
