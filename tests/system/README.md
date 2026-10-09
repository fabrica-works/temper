# temper's composed system world

`composed::World` drives the actual engine root, actual worker root and
smith's actual agent domain over fake LLM IO. The store and authenticated
person are jig's fakes. temper's fake forge serves both the engine's API
and the worker's git operations from one repository state. This follows
jig's `domain/testing.md`, sections 2 and 8, and temper's `testing.md`,
section 2.1.

The worker prepares an adopted repository before spawning the agent. The
process adapter mounts that prepared tree in smith's filesystem fake. The
fake LLM scripts only completion contents: smith emits its own calls,
turns, messages and terminal. Calls and turn credit cross smith's process
host, the worker host, the engine and its journal before returning.

This is the domain face of the composition. The ordinary process adapter
retains concrete charters, transcript turns and answers outside the domains
as a codec would, checking their opaque bytes at each crossing. It does
not test a deployed process, serialization or kernel containment. Existing
`world` fixtures retain narrower engine-to-smith forge and goal checks.

Focused cases cover a committed chat report, a durable host-call reply,
live human words, cancellation, a read-only adopted repository prepared
by the worker, park/resume from the committed concrete smith prefix, and
withheld store acknowledgements. Assertions independently read durable
task/call records and native smith observations. The worker answer must
match smith's actual result, turns and cumulative spend; every turn is
acknowledged and the process is reaped.

The fuzzy sweep runs 24 seeds twice, comparing durable rows, concrete
turns and crossing observations. Seeds vary fake LLM timing, store commit
delay, host calls and repository preparation. The composed clock advances
one millisecond per iteration; each smith iteration yields to the roots.
Both root configurations must admit their declared worst-case bounds.
