# Temper's composed system world

`composed::World` drives the actual engine root, actual worker root and
Smith's actual agent domain over fake LLM IO. The store and authenticated
person are jig's fakes. Temper's fake forge serves both the engine's API
and the worker's git operations from one repository state. This follows
jig's `domain/testing.md`, sections 2 and 8, and temper's `testing.md`,
section 2.1.

The worker prepares an adopted repository before spawning the agent. The
process adapter mounts that prepared tree in Smith's filesystem fake. The
fake LLM scripts only completion contents: Smith emits its own calls,
turns, messages and terminal. Calls and turn credit cross Smith's process
host, the worker host, the engine and its journal before returning.

This is the domain face of the composition. The ordinary process adapter
retains concrete charters, transcript turns and answers outside the domains
as a codec would, checking their opaque bytes at each crossing. It does
not test a deployed process, serialization or kernel containment. Existing
`world` fixtures retain narrower engine-to-Smith forge and goal checks.

Focused cases cover a committed chat report, a durable host-call reply,
live human words, cancellation, a read-only adopted repository prepared
by the worker, park/resume from the committed concrete Smith prefix, and
withheld store acknowledgements. Assertions independently read durable
task/call records and native Smith observations. The worker answer must
match Smith's actual result, turns and cumulative spend; every turn is
acknowledged and the process is reaped.

The fuzzy sweep runs 24 seeds twice, comparing durable rows, concrete
turns and crossing observations. Seeds vary fake LLM timing, store commit
delay, host calls and repository preparation. The composed clock advances
one millisecond per iteration; each Smith iteration yields to the roots.
Both root configurations must admit their declared worst-case bounds.

## Serial world measurements

Measured on 2026-10-09 through the shared build lock, unoptimized test
build, `nextest --profile measure -j 1 --ignore-default-filter`, selecting
all `temper-*-world` packages. Times sum individual test durations; the
whole workspace's parallel gate keeps its separate 15 s / 60 s limits.
The conformance world retains every one of its 365 commit cuts within
the decided 2 s / 20 s allocation (QUESTIONS.md, item 9).

| World | Focused tests / seconds | Fuzzy tests / seconds |
|---|---:|---:|
| conformance | 15 / 1.830 | 2 / 16.393 |
| engine root | 189 / 2.990 | 7 / 6.044 |
| forge client | 26 / 0.123 | 2 / 0.094 |
| forge connector | 46 / 0.231 | 1 / 0.024 |
| system | 16 / 0.315 | 1 / 0.357 |
| web | 27 / 0.117 | 2 / 0.980 |
| worker checkout | 31 / 0.594 | 2 / 0.928 |
| worker root | 13 / 0.071 | 2 / 0.116 |

The system world's added cases fit the allowed story growth of 0.5 s
focused and 3 s fuzzy. Shared test kits without test targets, including
`temper-world` and `temper-fake-checkout`, have no separate test duration.
