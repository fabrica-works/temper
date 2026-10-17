# Supplement jig-12.s1: a watch asked before signing in waits for it

Added by ruling 246, from the design answer of 2026-10-17T10:05Z
(jig `8c832e517`: `client/domain/client.md`, 8.1 and 12; temper
`c3d0c0b9`: `client.md`, 3.4, and `temper/03-client.md`, 03.5). Read
first: jig's `AGENTS.md`, `docs/development/workflow.md`; this plan's
README, "Rules of this plan" and "The run's steers".

Scope: jig's `client.md`, 8.1: "**A watch waits for a sign-in.** A
`Watch` while no sign-in is live (the link signed out, 8.2) is admitted,
its handle opening, and nothing leaves for the transport ... Once signed
in, the client opens each such watch under a fresh name; one on `me` is
the client's own (8.2), not a second."
Needs: nothing. It runs in a slot of its own, now, beside jig 21.3.
Order: it lands before jig 21.3's landing; 21.3 rebases onto it (temper
03.5's repin takes this step's landing without 21.3's).
Files, Shape, Test, Size, Proof and Don't: ruling 246's Decision,
point 3, as written (sites at jig `8c832e517`; follow each by its name).
Size: aim 60 added lines, stop at 100; past 100, a question.
Further users: a jig test or world that watches before signing in and
expects an `Open` (or an `OpenStream` followed by `Message::Watch`) is
adapted to open once signed in, counted in the Size, and named in the
commit body; one outside jig is the repinning temper session's.
temper: 03.5 takes this landing by a second repin commit, `Cargo.lock`
alone (`temper/03-client.md`, 03.5).
