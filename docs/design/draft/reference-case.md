# The reference case

Draft, 2026-10-21. One story that holds, once each, every move the
owners' process has made in its real runs up to 2026-10-21, including
that day's change of the process's shape. Temper must be able to run it.
The names are invented and the story is condensed. Each step names the
real episode it comes from. `temper-on-temper.md` is the analysis behind
it: its encoding (section 4), its grades (section 2) and what falls
short (section 6).

## 1. In one page

- **What it is:** a catalogue told as a story. Each step is something
  that has happened, or that the rules prescribe, and that temper must
  handle. The story is as small as it can be while keeping every
  mechanic.
- **The promise it tests:** temper is as flexible as a bare agent in a
  terminal with orchestration improvised around it, which is how the
  work runs today. On top of that, it gives the structure, the rails and
  the guarantees that the improvised way can't. So each step says what
  temper **must** do (the flexibility) and what it **adds** (the
  guarantee).
- **The cast:**
  - the person, who is today's user, with the director as their chat;
  - three owners: A, a Claude owner; B, a Codex owner from the old
    shape; and C, a fresh Codex owner that takes B's work on as a
    mission;
  - two repositories: kit, upstream, and app, which pins kit;
  - three workers: ws, the person's workstation, and two servers, s1
    and s2;
  - two plans: Claude's and ChatGPT's.
- **Four acts:**
  1. the old shape at work (section 4);
  2. the change of shape (section 5);
  3. the new shape, with A's milestones (section 6);
  4. two owners at once, after B wraps up and C takes its work on
     (section 7).
- **What temper lacks for it** (section 9):
  - temper-on-temper.md's gaps: adding a dependency to a live task,
    which jig has designed and is building; parking by the director;
    plan usage; worker health; an open finding at the tip;
  - from the new shape (temper-on-temper.md 6.8): an open finding at a
    change's head too; a repair that adopts work in flight; the cap
    against delegates' deadlines; a fresh run at a milestone's end; a
    goal stopped while its delegates go on; retiring a charter, and
    changing one under live tasks; a goal's notes and news passed to
    its successor.
- **Its readers:**
  - the web client's mockups, which show this story;
  - the lab, which runs it on the fake forge and the fake LLM, with
    each judgement scripted as its oracle;
  - the director, who maps what temper lacks onto owners.

## 2. How to read it

- **Numbers:** a section number inside a grade, as in "(design,
  6.5)", is temper-on-temper.md's. Any other number in parentheses is
  a step of this case.
- **Each step** tells what happens, and then gives two lists:
  - **Must:** the operations temper needs for it, each graded as
    temper-on-temper.md section 2 does: exact, sugar, design,
    conflict, plane, or gone;
  - **Adds:** what temper guarantees that the improvised way didn't.
- **Its source,** in brackets after the title:
  - R, the reference of 10-19 and 10-20, with its replay
    (temper-on-temper.md section 5) where there is one;
  - T, the change of shape on 10-21, with the director's decision
    number;
  - N, the new shape's first milestone on 10-21;
  - S, what the rules prescribe and has not been seen yet.
- **Time** is in hours from the start. On fakes, timers, deadlines and
  caps are scaled down.
- **The person may act directly or through the director.** Today
  everything goes through the director's chat. In temper the person
  could also stop, release, cancel or answer in the client. The story
  takes the director's path, and names where the person could act
  directly.

## 3. The cast and the start

- **The person:** directs the work, answers what is theirs to answer,
  and runs the machines and the sign-ins.
- **The director:** the person's chat, a conversation with a
  coordinator's charter, on Opus at xhigh. Its notes are the direction:
  the priorities, the standing rules, and what waits on the person. Its
  helpers are its delegates.
- **A,** a Claude owner on Opus at xhigh, owns the draft "Export". The
  draft spans kit and app, and lists no milestones.
- **B,** a Codex owner on GPT at high, owns the draft "Harness": kit's
  fake server, the flaky tests and the build path. It takes taste from
  one-shot Opus advisors, and brings its significant decisions to the
  director.
- **C,** a Codex owner like B, starts in act 4 with a mission: the part
  of Harness that the person's first priority needs.
- **Implementers, surveys and investigations** run on Codex, at medium
  or high by the task. Advisors are Opus one-shots. A tester is a Codex
  task.
- **The repositories:**
  - kit, the upstream;
  - app, which pins kit. app has a slow tier that runs after each
    landing, not in its gate.
- **The workers:**
  - ws, the person's workstation: small, its load the person's own, and
    the director's home today;
  - s1, a server;
  - s2, a server the person adds in act 1.
- **The accounts:**
  - the Claude plan, with two subscriptions, the second idle;
  - the ChatGPT plan, which has room to spare.
- **The rules:** the process documents in kit, conventions that every
  run reads. At the start they describe the old shape: an owner designs
  its draft, reports, then implements it in one long session.
- **At hour 0:**
  - the director runs on ws;
  - B runs on s1, with three Codex slots;
  - F is an open finding: a kit test that flakes. It is B's, and its fix
    f1 is a ready change.

## 4. Act 1: the old shape (hours 0 to 8)

### 4.1 A starts and designs (R 5.1)

The person: "Export's draft has landed; schedule it, priority 1." The
director:
- picks s1, which has three free slots, to be shared with B;
- makes A a Claude owner at xhigh;
- writes its brief: the draft at its commit, B's footprint in kit's
  fake, the budgets, the security rules.

A has Codex survey kit, lands its design as Markdown alone, and reports
in 15 lines. The director gives it a green light.

- **Must:**
  - delegate a goal with instructions, input, priority, host kind, pool
    and models (exact);
  - read which workers have room (sugar, temper-on-temper.md 6.7);
  - a question and its answer (exact);
  - the fast path for Markdown alone (exact).
- **Adds:**
  - authority bounds A's subtree;
  - spend rolls up to the goal.

### 4.2 A known flake at kit's tip (R 5.4)

A sends x1 (kit's new API) and x3 (a part of app on its own). x1 lands.
kit's after-landing tier then fails at the tip with F's signature. F is
B's, and f1 waits behind two other changes. The director:
- routes the failure to B rather than to A;
- raises f1's priority.

f1 lands a quarter of an hour later and the tier passes. kit takes
landings all along. x3 runs on.

- **Must:**
  - CI at the tip (exact);
  - priorities in the queue (exact);
  - a failure that matches an open finding attaches to it and leaves
    the queue running (design, 6.5). Today temper pauses kit's queue
    and asks for a repair, and f1 waits behind the pause (conflict).
- **Adds:** main is never red without anyone knowing.

### 4.3 The two-hour check finds s1 short (R 5.3)

The check's single report shows s1 short:
- 16 builds queued;
- a load of 1.45 a core;
- disk near its floor.

B's benchmark, which needs s1 quiet, has stalled for an hour holding
30 GB. The person added s2 this morning. The director:
- cancels the benchmark, freeing the disk for A's next repins;
- shrinks B's pool by one and lends that slot to A;
- moves B to s2 after its current round.

B's notes and branches follow it. Its benchmark data doesn't, and its
worktrees on s1 become cache.

- **Must:**
  - a timer (exact);
  - cancel (exact);
  - pools resized by amendment (exact);
  - amending a host kind, effective at the next run (sugar: amend
    doesn't name it);
  - reading load, queue and disk (design, 6.4: from the ground or an
    ops application);
  - carrying data that is neither git nor notes, and a benchmark's
    quiet machine (plane: hearth and the lab);
  - disk floors, build lanes, caches and memory caps (plane).
- **Adds:**
  - the report's spend per owner is computed, not estimated;
  - a moved owner leaves nothing running behind.

### 4.4 The pace (R 5.6)

The same report shows Claude spending eight times what the week can
sustain. The director asks the person, with its recommendation first:
move what needs no taste to Codex, and switch s2 to the second
subscription. The person answers hours later: "keep the pace while the
hardware allows; move what needs no taste". Then:
- the director amends A's and B's models;
- the person switches s2's sources;
- the answer goes into the director's notes, so the question isn't asked
  again.

- **Must:**
  - a question in the person's inbox (exact);
  - models amended (exact);
  - the holder's order of sources (exact);
  - a plan's windows, its resets and its sustainable pace as the
    account's facts (design, 6.3).
- **Adds:** the question waits in one inbox, not in two checks' reports.

### 4.5 Sign-ins and limits (R)

ChatGPT's sign-in on s2 expires: B's tasks wait, saying why, until the
person signs in on s2. Credentials are never copied between machines.
A's Claude reaches its five-hour limit: A's tasks wait for the reset.

- **Must:** an account's states hold the work they pay for, and name
  the act they wait for (exact).
- **Adds:** no session dies, so nothing has to be recreated afterwards.

### 4.6 ws hangs (R)

ws hangs for four hours. Three of A's questions wait, since the director
lives there.

After the reboot:
- two resumed copies of a session act beside their originals;
- a completion checker is orphaned;
- a Codex run hangs on its stdin;
- an implementer stalls with no output.

That same day, an implementer printed a session token that it found in
the build queue's environment.

- **Must:**
  - the chat's run placed again after its worker's grace (exact);
  - a stalled run held past its bound and retried (exact);
  - one run per task, the lost attempt fenced before the next (gone).
- **Adds:**
  - no single point;
  - nothing resumes twice;
  - forge credentials never on a worker. The build queue's environment
    is hearth's (plane).

## 5. Act 2: the change of shape (hours 8 to 12)

### 5.1 The person holds the director and rewrites the rules (T 334-336, 351)

The person stops the director's chat for four hours and rewrites the
process documents into the new shape:
- an owner takes one mission, a draft or part of one with a defined
  end, in at most four milestones, and then stops; the next mission
  gets a fresh owner, and no owner is standing;
- a mission is reached through milestones of 2 to 3 hours, under a
  4-hour cap;
- each milestone has a sub-owner;
- the plan comes first;
- the owner starts a fresh session at each milestone's end;
- a QA pass runs beside the next milestone;
- the director compacts on a budget and starts fresh every third check.

One rule goes: owners no longer check shared version numbers. A asks
the director about x3's scope, and the question waits all this time.
The person then releases the chat.

- **Must:**
  - Stop and Release on the chat (exact);
  - questions wait in its inbox (exact);
  - the rules land as conventions that every later run reads (exact);
  - the director's next run starts fresh and reads them (exact). Its
    compaction budget and fresh contexts are its charter's and the
    deployment's resume limit (gone).
- **Adds:**
  - no message telling anyone to re-read a rule;
  - a held owner needs no notice that a rule was dropped.

### 5.2 Every owner but A pauses (T 332, 335)

The person: "pause every owner but A; I'll go through each one's work
and say what resumes". The director can't stop B. It proposes the stop,
and the person accepts. Then:
- B's run ends;
- its running implementer finishes into its worktree, and nothing acts
  on the result;
- h1, already in kit's queue, lands.

The person also asks that the old process's leftovers stop and never
start again. Two of the old shape's helper tasks are cancelled, and
their charter is retired.

- **Must:**
  - Stop, which is the person's (exact);
  - the director parking on the person's word (design, 6.2);
  - delegates and queued changes going on under a stopped goal (to
    check, 6.8);
  - retiring a charter that still has tasks (to check, 6.8).
- **Adds:** a stopped goal's news waits for its release, so nothing is
  relayed by hand. Its reaching a successor instead is 7.1's.

### 5.3 A moves to the new shape (T 340, 341)

The director amends A's instructions to a new brief:
- x3 finishes the old way, outside any milestone, under the person's
  cap of two hours ("if it doesn't finish, we evaluate together");
- then A's milestone plan.

A prunes its notes into an archive.

- **Must:**
  - instructions amended (exact);
  - a deadline on x3 whose hold goes to the person, as the holder of
    its escalations (exact);
  - the charter's new run policy reaching a live goal (to check, 6.8).
- **Adds:** the amendment is in A's history, with what it replaced.

### 5.4 The person changes direction (T 342, 343)

Twenty minutes later the person writes: "the end-to-end story first;
cut it to one path; placeholders are fine; move whatever it needs into
A, so it waits on no paused owner." So:
- A's half-written plan is deferred, kept in its notes;
- x3 can't land within its cap, so A stops it on its branch: pushed,
  kept, named in its notes;
- A may change B's files where the story needs them, and lists each
  change under B for B's rebase.

- **Must:**
  - amendment (exact);
  - cancel, with the work kept on its branch (exact);
  - grants on another owner's files as a convention, with footprints
    as detection (exact; enforced grants are sugar, 6.7).
- **Adds:** B's news of the overlaps, kept for its release, replaces
  the list of hunks.

### 5.5 A's plan and its first milestone (T 343, N)

A's plan, in 15 lines:
- a nearer end: Export needs more than four milestones, so this mission
  ends at the end-to-end story, and the rest of Export goes to a later
  mission;
- M1, the path through app and kit's fake;
- M2, the rest of the story;
- each 2 to 3 hours;
- an ask for B's files.

The director gives it a green light, with every ask granted. A hands
over to a fresh session. The fresh session starts M1's sub-owner, A1,
with a 4-hour deadline.

- **Must:**
  - a question and its answer (exact);
  - a mission's size: a count of milestones by convention, or a goal's
    deadline at about 16 hours as its rail (sugar);
  - a sub-owner delegated with a deadline (exact);
  - an owner resumed within a milestone and fresh at its end (sugar,
    6.8: a charter policy, or a call for a fresh next run);
  - a sub-owner that can't start one of its own (exact, a rail:
    authority's depth).
- **Adds:** the handover routine goes: one run per task, so there is no
  old session to stop.

### 5.6 The person's priorities, misread (T 344-347)

The person writes the phase's priorities "for information":
- 1, the end-to-end story on fakes;
- 2, real backends;
- s1 and s2 preferred for priority 1.

The director starts a helper to act on them. The person: "that was
just informative; don't act on it". The helper is cancelled before it
lands anything, and the director records the rule.

Meanwhile the director's first two fresh starts fail, because a timer
of its own kept its session busy.

- **Must:**
  - cancel a delegate (exact);
  - the direction as the chat's notes (exact);
  - priorities and preferred host kinds (exact).
- **Adds:** timers are the task's, outside its run, so they can't block
  a fresh start (gone).

## 6. Act 3: the new shape (hours 12 to 17)

### 6.1 M1's design and its tasks (N)

A1 sends two Codex surveys at once, lands M1's design as Markdown, and
reports five departures from A's read. A gives a green light with three
amendments, one of them "if cheap". A1 then sends:
- a1, a2 and a3 together, on the three slots s1 lends;
- a4, the story, after a1 and a2.

- **Must:**
  - parallel delegates (exact);
  - the fast path (exact);
  - a question and its answer (exact);
  - a batch with its dependencies (exact);
  - a pool (exact).

### 6.2 A blocker in a paused owner's crate (N)

a3 finds kit's fake answering wrongly. The fake is B's. A1's fix, e1,
changes six public signatures, though nothing outside the crate calls
them. Then:
- A1 asks A, and A asks the director, since the change touches a
  stopped owner's public API;
- e1 waits for the answer: a checkpoint;
- a3 now waits on e1's landing and on app's repin to it.

- **Must:**
  - a proposal the director decides (exact);
  - a checkpoint (exact as a convention; sugar as a gate an ancestor
    judges, 6.7);
  - a dependency added to a live task (jig's tasks.md 7, designed and
    being built: exact once built).
- **Adds:** the added dependency is in the tree, visible and checked
  for cycles. a3 stops going on rather than being sent back.

### 6.3 Landing on a tired machine (N)

a1 and a2 land as one change, to save a gate. CI fails four times on
the way:
- once for real: the change moved a memory figure, and is repaired;
- three times because of s1:
  - its CPU clock fell under sustained builds;
  - F2, a known flake on slow runners;
  - a global timeout.

app's queue holds five changes at its peak. A1 sends B two findings:
the clock, and F2's rate.

- **Must:**
  - CI at each change's head, and a real failure repaired (exact);
  - a failure that matches an open finding, or comes from a runner
    flagged unhealthy, runs again rather than starting a repair
    (design, 6.8: 6.5's rule at the head, with 6.4);
  - findings with signatures (plane: the lab);
  - news to a stopped goal (exact);
  - fewer, larger landings (exact, a judgement);
  - each change's wait measured (exact).
- **Adds:** CI runs on each head at once, before the queue.

### 6.4 Red main (N)

After a1 and a2 land, app's slow tier fails at the tip: a1 changed what
the story sees, and the gate doesn't run that tier. The director rules:
a4 fixes it, or a smaller fix lands within ninety minutes.

- **Must:**
  - CI at the tip pauses app's queue and asks for a repair (exact);
  - the repair adopts a4, which is already in flight (design, 6.8);
  - a timer for the interim fix (exact).
- **Adds:** nothing lands on a red main.

### 6.5 A budget question reaches the person (N, S)

a4's story takes 70 s against its suite's budget of 60 s. The question
goes up four levels: A1 asks A, A asks the director, and the director
asks the person, since budgets are theirs. The recommendation: no
raise, make it cheaper. The person agrees.

a5, a Codex investigation at medium, finds no cause. Raised to high, it
finds none again. An Opus advisor reads its report and its evidence,
and names the cause.

- **Must:**
  - an escalation up four levels to the person's inbox (exact);
  - a delegate's effort raised for its next try (exact);
  - a one-shot advisor (exact).
- **Adds:** the question's path is in the tree, with each level's
  answer.

### 6.6 The director's word, late (N)

The director is unsure whether e1 is the person's to decide, since B's
stop was theirs, so it asks. The answer comes an hour later: yes, and
tell whoever takes B's work on. Then:
- e1 lands in kit;
- app's repin to it lands;
- a3 and a4 land.

The tip turns green, and the interim fix isn't needed.

- **Must:**
  - dependent changes across repositories (exact);
  - a landing that releases what waits on it, with no relay (exact).

### 6.7 The cap (S)

At four hours, a5's fix is still running. M1 is held. A1 escalates
through A to the director with:
- what has landed;
- the cause: the late word and the tired machine;
- the options: split, or more time;
- its recommendation: split, so that what has landed is M1 and a5's
  fix opens M2.

The director splits it, without asking the person.

- **Must:**
  - a deadline holds the task and tells its requester (exact);
  - the decision as an amendment, or as a new sub-owner (exact);
  - but jig holds the running delegates too, since their deadlines can
    be no later than the sub-owner's, while the cap lets what runs
    finish (conflict, 6.8: it needs a grace).
- **Adds:** the cap is a rail, not a rule to remember.

### 6.8 M1's end and its QA pass (S)

A1 updates STATUS and reports in ten lines, and A passes the report
on. A's next run is fresh, and starts M2's sub-owner, A2, at once.

A1 stays on for M1's QA pass. Its tester, a Codex task, reads the
design only and writes adversarial tests. It finds:
- a bug: fixed, with its regression test in the lowest world;
- a test that misreads the design: dropped, and the design made
  clearer;
- a gap A1 accepts: ignored, and listed.

A1 lands the fixes and closes with its note.

- **Must:**
  - a message to the requester (exact);
  - the next milestone's sub-owner delegated on that message (exact);
  - a tester without forge effects (exact, a rail: it can't land).
- **Adds:** the tester's tests land only through A1's change.

## 7. Act 4: two owners at once (hours 17 to 24)

### 7.1 B wraps up, and C takes its work on (T 347, 351, 353; R 5.5)

The person: "B wraps up; the part of Harness that priority 1 needs
becomes a mission, on s2." The person releases B, and the director
amends its instructions to a wrap-up: land what is held and ready,
write the closing note with what is left, and stop. B lands h2, names
its parked benchmark branch in the note, and completes.

The director starts C, a fresh Codex owner on s2. C's mission is the
part of Harness that priority 1 needs, with its end, and its input is
B's notes and closing note. C needs what was kept for B:
- A's changes in B's files;
- e1's approval;
- the clock and F2 findings.

C proposes one milestone, N1, which it drives itself: B's old plan,
reordered toward priority 1, with a date its mission must meet. It
takes its API's shape from an Opus one-shot. The director gives N1 a
green light.

- **Must:**
  - Release, the person's (exact);
  - amendment (exact);
  - a goal that completes with what is left in its result (exact);
  - a new goal with the last one's notes and closing note as its input,
    and the news kept for B reaching C (design, 6.8: a goal's notes
    and news are its own);
  - a goal's deadline (exact);
  - a Codex owner taking taste from an advisor and bringing its design
    to the director (exact).
- **Adds:** what B leaves is its result, which C takes as input, so it
  isn't a file someone must remember to read.

### 7.2 Two owners, one repin (R 5.2)

M2 needs C's kit change k2: A2's a6 depends on k2's landing. Both C's
c2 and A2's a6 need app's repin of kit. The director orders a single
repin:
- C's repin goes first and carries both adaptations;
- if it is more than an hour out, A2 repins first instead.

It lands early, and a6 starts with no relay. A2's branch then conflicts
in app's lock, and a conflict resolution takes main's lock and updates
again. The director introduces A2 and C to each other, to settle k2's
API between them.

- **Must:**
  - a dependency on another goal's task (exact);
  - ordering by priority and a held branch (exact);
  - a timer (exact);
  - a conflict resolution (exact);
  - an introduction (exact).
- **Adds:**
  - a landing releases what waits on it at once;
  - a conflict becomes a task.

### 7.3 The end (S)

M2 lands. It is A's last milestone, so there is no handover: A2's
report, its QA pass, then A's closing note, and A completes. The note
gives:
- each milestone's time against the cap;
- how the plan moved, and why;
- the QA passes;
- the spend;
- what is left of Export, for the next mission's brief.

C ends N1 with its own note. The director compares the notes and
proposes one change to the process documents.

- **Must:**
  - results, and a goal that completes when its mission does (exact);
  - notes (exact);
  - spend per goal (exact).
- **Adds:** cost per owner, per milestone and per landing, which the
  improvised way can't compute.

## 8. The catalogue's index

Every practice in temper-on-temper.md 4.6, and every move the change
of shape added, with the steps that hold it.

| Practice | Steps |
|---|---|
| The director as the person's chat, its notes, its helpers | 4.1, 5.1, 5.6 |
| Starting an owner: brief, input, priority, host kind, pool, models | 4.1 |
| An owner per mission, at most four milestones; a nearer end in the plan | 5.1, 5.5 |
| An owner wrapped up; its work taken on by a fresh owner, with its notes | 7.1 |
| A Codex owner, its advisor, its decisions to the director | 7.1 |
| Implementers, surveys, investigations, advisors | 6.1, 6.5 |
| A tester that can't land | 6.8 |
| Design report and green light, with amendments | 4.1, 6.1 |
| Milestone plan first, checked like a design | 5.5, 7.1 |
| A sub-owner per milestone, under a deadline; a single milestone driven by its owner | 5.5, 7.1 |
| A fresh owner run at a milestone's end | 5.5, 6.8 |
| The cap: held, escalated, split | 6.7 |
| A milestone's end: report, STATUS, the next started, QA beside it | 6.8 |
| Footprints, overlaps, grants on another owner's files | 5.4, 6.2, 7.1 |
| A change to another owner's API, as a proposal | 6.2 |
| Checkpoint before landing | 6.2 |
| Dependencies known up front | 6.1 |
| A dependency found later, added to a live task | 6.2 |
| A dependency on another goal's task | 7.2 |
| A landing releasing work, with no relay | 6.6, 7.2 |
| Ordering: priorities, a held branch, timers, deadlines, splits | 4.2, 6.4, 6.7, 7.2 |
| Repins, one carrying two owners' adaptations | 6.6, 7.2 |
| Conflict resolution | 7.2 |
| `land`: CI at the head, repairs, the fast path | 4.1, 6.3 |
| Landing throughput: the peak, fewer and larger landings | 6.3 |
| After-landing: a known flake at the tip | 4.2 |
| After-landing: a regression at the tip, repaired by work in flight | 6.4 |
| Failures from the machine or a known flake at the head | 6.3 |
| Slots, throttling and lending | 4.3, 6.1 |
| Relays and introductions | 7.2 |
| The two-hour check and its report | 4.3, 4.4 |
| Escalation to the person: the pace, a budget, a call the director doubts | 4.4, 6.5, 6.6 |
| Learning: an answer kept as a standing rule | 4.4, 5.6 |
| Claude for taste, Codex for the rest, its effort by the task | 4.4, 6.5 |
| Logouts, limits, switching subscriptions | 4.4, 4.5 |
| Pacing against a plan's window | 4.4 |
| Parking and pausing an owner; releasing it | 5.2, 7.1 |
| Moving an owner between machines; placement by load and disk | 4.3 |
| A worker lost; duplicates, orphans and hangs | 4.6 |
| Credentials and secrets on workers | 4.5, 4.6 |
| Disk floors, caches, build lanes, benchmarks | 4.3 |
| Rules changed mid-flight; a rule dropped | 5.1 |
| The coordinator held while the rules change | 5.1 |
| A live owner moved to a new shape, its work in flight capped | 5.3 |
| A deadline whose hold goes to the person | 5.3 |
| A plan deferred for a new direction; scope cut; placeholders | 5.4 |
| Work moved off paused owners into the live one | 5.4 |
| Retiring the old shape's charters and their tasks | 5.2 |
| An informative message misread; a delegate cancelled | 5.6 |
| Resumes one at a time, each to a mission or a wrap-up, toward priority 1 | 7.1 |
| Closing notes with what is left, compared; spend per owner | 7.1, 7.3 |

## 9. What the case asks of temper

From temper-on-temper.md section 6, unchanged:
- **Adding a dependency to a live task** (6.2): designed in jig's
  tasks.md 7, and being built.
- **The director parking an owner** (5.2), on the person's word or for
  credit.
- **A plan's usage** in the account's facts (4.4).
- **Worker health:** load, queue and disk, read and acted on (4.3).
- **An open finding at the tip** does not pause the queue (4.2).
- **Smaller ones:** reading the fleet's free slots (4.1); amending a
  host kind (4.3); a gate an ancestor judges (6.2); enforced grants
  (5.4).

New with the change of shape and the new shape's first milestone
(temper-on-temper.md 6.8):
- **An open finding at a change's head,** and a runner flagged
  unhealthy, run CI again rather than start a repair (6.3). In the
  first milestone, eight of nine failed gates were the machine or a
  known flake. In temper each would have been a repair with nothing to
  fix.
- **The repair adopts a change already in flight** when CI fails at
  the tip (6.4).
- **The cap against delegates' deadlines:** what runs at the cap
  should finish, but jig holds it with its parent (6.7).
- **A fresh run at a milestone's end:** a charter that resumes within
  a milestone and starts fresh after it (5.5).
- **A goal its party stopped:** whether its delegates go on and its
  queued changes land, as pausing needs (5.2).
- **The old shape retired:** a charter withdrawn while tasks still use
  it, and a charter's new policy reaching live tasks (5.2, 5.3).
- **A goal's successor:** a goal's notes are its subtree's, and its
  kept news its own. A fresh goal that takes on an ended one's work
  starts with neither (7.1).

## 10. Left out, and why

- **Real backends:** the case runs on the fake forge and the fake LLM,
  and a real backend changes no step.
- **The first run's own pages** (the box, the demo), which are
  first-run.md's.
- **The external QA loop fed by milestones,** which isn't agreed.
- **hearth's work** and the ground's own repairs (crash capture,
  remounts), which belong to those planes. The case only shows where
  temper meets them.
- **Design sessions,** which are inputs here: a landed draft, landed
  rules.

## 11. Open questions

- **The person's direct acts:** which of the director's acts in this
  story the person should rather do in the client (stop, release,
  cancel, answer), and how the director proposes them.
- **The oracle:** which judgements the lab scripts word for word, and
  which it checks only by their effects.
- **The scale on fakes:** how far timers, deadlines and caps shrink
  without changing the order in which things happen.
