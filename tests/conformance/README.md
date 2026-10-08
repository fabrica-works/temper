# Temper's conformance world

The concrete engine root runs on jig's conformance harness and outside referee.
Store rows come from `jig-fake-store`; workers retain their attempts, charges and
unacknowledged calls across engine crashes. Restoration uses the engine's real
bounded load requests. The adapter's `Resume` probes drive the root between
external inputs; its bookkeeping adds no task or forge decisions. The fixture
uses a one-second landing queue window and write lifetime, with restore pages
bounded to 32 rows and 64 KiB. Each shared scenario grants its real push
workspace and the effects it exercises; issue projections are not configured
for stories without goals.

The independent fake forge receives translated native API operations. Its
observations retain actual pre-write state, actor ownership, server replies and
native changes. Released writes require durable outbox entries and attempts.
The root contributes no effect verdicts. Recovery declarations follow the
native client's classes and temper's `forge.md`, section 6.

| Shared scenario | Temper binding |
|---|---|
| LateCopy | An idempotent commit status set and a retained transport copy |
| OtherHand | A real producer push, an opened pull, a decided merge and another actor's push |
| UncertainRestarts | An idempotent status flight whose absolute attempt deadline survives three cold starts |
| ChangedJudge | Real CI changes after a merge decision; the committed flight remains decided and another change cannot merge |
| NarrowingPolicy | An authenticated owner adds a requirement; a committed status flight resolves while a pending proposal and a fresh call are checked again |
| StandingTask | Recurring agent batches under successive funding periods, with duplicate period notifications |
| RunBudget | Twelve independently charged completions sharing one run allowance |
| ShrinkingPool | Not applicable: temper's forge has no pooled holds |

Focused tests run the baselines and three selected commit cuts per applicable
story. The fuzzy test crashes every applicable baseline at every durable commit.
Both suites run the same outside referee after every harness iteration.

The serial `measure` profile budgets are 1.5 seconds focused and 8 seconds fuzzy
(jig extraction QUESTIONS.md, item 9). Measurement runs retain every cut and
the same outside observations as the gate.
