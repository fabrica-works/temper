# Performance

Provisional, 2026-10-03, revised 2026-10-14. Whether the memory strategy of skein's
`docs/foundation/programming-model.md` (section 6: counted entities,
owned bytes, the worst case checked at startup) costs temper anything
that matters, where it could, and what to do about each case. Why the
strategy is what it is, what it gives up, and the byte budget it keeps
in reserve are in skein's `docs/foundation/notes.md`. This document
reviews the strategy against temper's workload; it adds no mechanics of
its own. Each fix belongs in the document or crate it changes, and moves
there when it is made.

## 1. In one page

- **No significant problem.** temper's time goes to LLM calls, which take
  seconds each, and to forge calls under a rate limit, at modest
  concurrency. Each agent run is a process of its own, smith's agent run
  as temper's `agent` command (`domain/worker.md`, section 6;
  protocol.md, section 4), so one process holds one run's sessions. What
  the memory strategy costs is microseconds to milliseconds per turn.
- **Four things are worth knowing,** each fixable by a contained change
  that touches no step code built so far:

  | Concern | What it costs | Fix | Effort | When |
  |---|---|---|---|---|
  | Bytes of unknown length (3) | quadratic copying, if done naively | two containers | small | with the forge and git components (protocol.md, section 12) |
  | Copy at emission, every turn (4) | peak memory, up to three copies of a transcript | smith's, for its session; in temper, counted by jig's components | none of temper's | done where each copy is made |
  | The general allocator (5) | page faults, fragmentation | glibc tuning, or another allocator, in the shell | small | after measuring |
  | A worst case of maximums (6) | memory headroom, not speed | the byte budget of notes.md | moderate | only if it bites |

- **Only the first needs action, with the protocol layer's own
  components.** The others are smith's and jig's accounting, or
  measurements to take once `temper serve` runs.

## 2. The workload

- **The agent** is smith's: it runs sessions, each turn sending the whole
  transcript to the provider and receiving an answer streamed in pieces.
  Its costs are smith's to account for (smith's design).
- **The worker** runs agent processes and git, whose output its git
  component parses (git.md). A resumed run's window comes down once, at
  its start, and the worker holds at most one per slot; a run's turns
  kept unacknowledged are bounded per run (`domain/worker.md`, section 8).
- **The engine** is the long-lived process. It holds a bounded working set
  of tasks and of the forge's live resources, read a page at a time, not
  the forge's contents. A turn's bytes cross it once per turn: up jig's
  channel, into jig's store's transcript file, and out to the clients that
  watch the run, each bounded by a turn's limit.

All three wait far more than they compute. Copying a megabyte takes
around a tenth of a millisecond; an LLM call takes seconds.

## 3. Bytes of unknown length

A `Box<[u8]>` cannot grow, `List` allocates its whole capacity when it is
made, and `Writer::new` wants the final length up front. That suits bytes
whose length is known before they are made, which is most of them. Some
arrive in pieces with no total announced:

- a forge's answer arrives as an HTTP body, which may be chunked, and
  whose strings the forge component cuts to the domain's limits as they
  stream (forge.md, 3.3);
- a git invocation's output arrives as pipe reads, of which the git
  component keeps what it parses and a bounded tail for diagnostics
  (git.md, section 6);
- a provider's streamed answer and a command's output in a run are
  smith's, with the same need.

Joining by making a new box for each piece copies everything received so
far each time, which is quadratic: a 10 MB build log in 64 KB pieces
copies about 800 MB, and a 32 KB answer in 1,000 deltas about 16 MB.
Making a `List<u8>` at the cap instead allocates the cap for every call in
flight, however short its answer.

Domain code builds bytes in one pass: it adds up the lengths, then
writes once. The pieces that need joining all live in the protocol
layer's components. Below each machine,
the carry-over is already safe: skein's lib, which temper uses, has an
`Intake`, a buffer allocated once at its cap that delivers each demand as
a box of exactly its length (programming-model.md, 4.3).

**Fix: two containers,** each with its `worst_case`, for what happens
above the intake. Where skein's lib has them when the forge and git
components are built, those are used; where it does not, each is a
module of the component that needs it, and moves to skein's lib with no
change to its callers when lib gains it (protocol.md, section 11):

- **A capped buffer that grows by doubling.** It starts small. A write
  that does not fit grows it to twice its size, or to what the write
  needs if that is more, but never past the cap; a write past the cap is
  refused whole, writing nothing, which the caller handles as any other
  limit. Finishing trims it to a box of exactly its length.
  - Each byte is copied in once, about once more on average as the buffer
    grows, and at most once more by the trim: linear in all.
  - While it fills it holds at most twice what it has received, and never
    more than the cap. Its worst case is the cap, as for a buffer made at
    the cap, but a short answer costs only its own length.
  - A list of pieces joined once at the end copies each byte only once,
    but a provider's deltas are a few bytes each: it would make one
    allocation per piece, and need a cap on pieces that is hard to
    choose.
- **`ByteRing`,** a container lib does not have yet: a buffer of fixed
  size, made once, that keeps the last N bytes written, each write
  overwriting the oldest, where an intake refuses past its cap instead. A
  git invocation's diagnostic output goes through a ring of its tail's
  bytes, and what falls out of the ring is counted as dropped. At exit
  the ring is read out, oldest first, as the tail (git.md, section 6).

Both are small modules, and they change no existing code.

## 4. Copy at emission, every turn

A step function that hands bytes to its protocol layer copies them as
programming-model.md (6.2) says it must: the domain keeps its state, and
the protocol layer owns the copy. Where the bytes are a whole transcript
sent on every turn, the copying grows with the square of the turns, and
while a call is in flight the bytes are held up to three times: in the
domain, in the request, and encoded on their way out.

- **The time does not matter.** A session of 200 turns whose transcript
  grows to 1 MB copies about 100 MB in all, tens of milliseconds over a
  session that lasts tens of minutes.
- **The peak memory does,** and is counted where the copy is made: each
  receiver's worst case counts, for each request in flight, the bytes it
  holds and their encoding, and encodes in pieces as io grants room where
  a body may be large.
- **The agent's session is smith's.** The whole transcript sent to a
  provider every turn is smith's session's, and its accounting is
  smith's design's, not temper's.
- **In temper's engine, a turn is copied a bounded number of times,** each
  by a component of jig's that counts it: jig's channel component as it
  decodes the turn, jig's store as it writes the transcript's frame, and
  jig's clients' component as it sends the turn to a run's watchers. Each
  copy is bounded by a turn's limit, and none outlives its request. A
  resumed run's window is encoded once, into its assignment, within the
  resume limit (jig's `protocol/channel.md`, 6.1). temper's own routing
  adds no copy of a turn: temper-engine-smith reads a run's result and a
  host call's input, not its turns (llm.md, section 4).

## 5. The general allocator

The memory strategy accepts the general allocator in the hot path, and
notes.md counts it among what the strategy gives up: allocation time is
not constant, and fragmentation can push the resident size above the
live bytes. With glibc:

- an allocation above the mmap threshold (128 KB to start) is its own
  `mmap`, page faults as it is filled and a `munmap` when freed;
- after such a free, glibc raises the threshold to its size, up to 32 MB;
  large boxes then come from the heap, where a long-lived process making
  boxes of mixed sizes can fragment: the heap returns memory to the
  system only from its top, so a small box that outlives large ones
  freed below it keeps them resident. They are reused by later boxes
  that fit, so it is not a leak, but the resident size can stay well
  above the live bytes.

The engine is the process this could affect; an agent process lives for
one run. **Measure first,** as programming-model.md (6.3) says: the
simulator's counting allocator gives the live bytes, and a load run of
the real shell gives the resident size to compare them with. **If the
gap matters,** in order:

1. tune glibc with no code at all, through its tunables in the service's
   environment: `GLIBC_TUNABLES=glibc.malloc.mmap_threshold=…:glibc.malloc.trim_threshold=…`.
   A fixed mmap threshold stops the raising, so large boxes always have
   mappings of their own and go back to the system when freed, and only
   small ones share the heap;
2. or make the same settings once at startup with `mallopt`, through
   `libc`, which the shell already depends on, so they travel with the
   binary;
3. and have the shell call `malloc_trim(0)` when it is about to block
   with nothing to do, which returns free pages from anywhere in the
   heap, not only its top;
4. or give the shell another `#[global_allocator]` (jemalloc, mimalloc),
   which keeps allocations of a size together and returns unused pages
   over time. It is a one-line change, but it adds a dependency that
   programming-model.md (2.1) does not allow today, so taking it starts
   with a change to that rule, in skein.

None of these touches step code: the binary chooses the allocator, and
step crates only allocate.

## 6. A worst case of maximums

The worst case (programming-model.md, 6.3) is every entity at its cap at
once. Transcript sizes have a long tail: most sessions stay small and a
few approach `session_bytes`. With the copies of section 4 a process sets
aside about sessions × `session_bytes` × 3, so eight sessions of 4 MB come
to about 100 MB. That costs headroom, not speed, and it bites only if one
process should run many sessions with contexts of a million tokens at
once.

**Fix, if it bites:** the byte budget, the fallback notes.md keeps open
for a worst case too large to provision.

- For io's queued output it is ordinary backpressure: io grants room only
  within a global budget, as notes.md describes, so pressure turns into
  the backpressure chain (programming-model.md, section 7).
- For sessions it is harder. If sessions charged a shared pool as they
  grew, one could run out partway through its work because of the others,
  which is the refusal in the middle that programming-model.md rules out
  (section 7). Done right, a session is granted bytes from the pool
  when it opens, and perhaps again between turns, where waiting is an
  ordinary state. That touches the agent domain's top level, the session
  crate and their worlds: a few days' work, after a design decision.

It is probably never needed: a process holds one run's sessions, and its
limits are that run's. Sessions are smith's; in temper, the worst case
that matters is the engine's, summed over its root and every component of
its protocol layer and checked whole at start (configuration.md,
section 7).

## 7. What does not cost

- **Allocating slabs, queues and lists up front.** Their capacities are
  counts, and their items are small; what the items own is allocated as
  it is needed.
- **B-trees instead of hash maps.** Logarithmic lookups over hundreds or
  thousands of entries.
- **Moves between layers.** A payload crossing a boundary moves a pointer,
  not its bytes.
- **Copying out of io's buffers.** One copy of each byte received, which
  exact-size reads remove later without touching anything above io
  (programming-model.md, section 8).
- **Retiring at the reclaim point.** A mark during the iteration and a
  push onto a free list at its end.
