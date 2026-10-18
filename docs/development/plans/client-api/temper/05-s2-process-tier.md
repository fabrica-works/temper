# Supplement temper-05.s2: the whole process tier passes

Added by ruling 367 (the director's request of 2026-10-18, for
forge-setup's Done when 4, which runs the whole tier within 180 s). Read
first: `temper/05-s1-landing-story.md`, whose Read first, Proof and Don't
hold here unchanged.

Scope: `temper_serve_answers_a_signed_delivery_204`
(`crates/temper/tests/process.rs:309`) passes again on `main`, and so does
each test of the tier that the 13:49Z run of `scripts/proof-one-process.sh`
did not reach. At `main` `acc7496e2`, run once with the whole tier, it
failed at 30.077 s: "at 29999 ms, the referee failed the real loop:
webhook delivery or drain did not finish" (`support/webhooks.rs:115`:
the POST was not answered, or `temper serve` had not exited after its
`SIGTERM`); four tests did not run.

Needs: nothing. It runs off jig's chain, beside temper 06 and jig 19.

Shape:
1. **Every failure first:** the whole tier once with `--no-fail-fast`:
   `heavy --gate sh -c 'export NEXTEST_DOUBLE_SPAWN=0 && cargo build -p
   temper --bin temper && cargo nextest run -p temper --profile process
   --no-fail-fast'`. Each failing test, with its first failure line, is a
   cause of this step.
2. **Focused, ruling 229:** the signed delivery alone, traced at the
   lowest layer that shows which part stops: whether serve printed its
   link, the POST's status and when, and after the `SIGTERM` which of
   serve's components has not closed (the webhooks listener, the clients'
   HTTP server, the embedded worker, the engine). The trace's lines are
   the implementer's choice, and go before the commit.
3. **If two traced runs do not name it,** one run of the test alone at
   each first-parent commit of `git log --first-parent 7f5ce036f..origin/main --
   crates/` (`7f5ce036f` last changed the referee): the first that fails
   names the change. By their subjects, `3262df8f9` (temper-04.s1,
   "Finish held HTTP calls once their clients component closes":
   `server.rs`, `stop.rs`, `system.rs`) and `63f6174dc` come first.
4. **Repair** where the log names it: temper's production code or the
   tier's support, one commit for each cause, as 05.s1's Shape says. A
   cause in jig (its forge kit hosts the webhooks listener,
   `crates/temper-service/src/forge.rs:77`) stops the step, with the log's
   cause line.

Bound: three focused runs and 40 minutes from the unblock; past it, the
report gives what the trace shows.

Proof: 05.s1's: the landing story alone, then the whole tier once, `heavy
--gate scripts/proof-one-process.sh`, within 180 s.

Size: about 100 production lines. Past 200, it asks.
