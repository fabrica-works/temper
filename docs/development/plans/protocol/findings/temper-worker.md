# Audit: temper-worker

2026-10-14, read only, by a sub-agent of the planner, at temper `cc2697cf` with jig `d91a89c4`, smith `dfc0fe57` and skein `b5d3a2c3` (temper's pins; jig's, smith's and skein's paths are relative to their pinned checkouts). Evidence the plan cites (development.md, 4.1 and 5.4): where it and landed code differ, the landed code wins. The design gaps it found are closed by the Markdown landing "Design: close the gaps the protocol plan's code audit found"; `README.md` here maps each finding to its increment.

I found 13 differences between the code and the design. The two that block the git component's steps are in skein-io: its file requests cannot rename across directories, and cannot remove an entry inside a directory opened while running. On the worker root, the landed work already names repositories by tag, but saved work and a failed preparation do not yet. Nothing was edited; no cargo was run. Paths are relative to `~/src/rust/temper` unless a crate path is given; skein paths are at b5d3a2c3, jig at d91a89c4.

## 1. `temper-worker-domain-checkout` against git.md section 4
- **`Op`** (git.rs:58-92) matches git.md exactly, variant for variant and field for field:
  - `Make{workspace: Token}`, `Clone{at: Place, remote: Box<[u8]>, identity: u32}`, `Fetch{at, remote, want: Want, identity}`
  - `Create{at, remote, branch, commit: Commit, identity}`, `CheckOut{at, commit}`, `Merge{at, theirs: Commit}`
  - `Commit{at, parent, merging: Option<Commit>, title, body, identity}`, `Push{at, remote, commit, branch, expected: Option<Commit>, identity}`
  - `Place{workspace: Token, repository: Box<[u8]>}` is at :50. `kind()` is at :175 and `is_remote()` at :190 (true for Clone, Fetch, Create, Push).
- **`Want`** (:96) is `Branch{branch}`, `Commit{commit}`, `Default`.
- **`Done`** (:108) is `Merged`, `Conflicted{files: Box<[Box<[u8]>]>}`, `Succeeded`, `Fetched{commit}`, `Committed{commit}`, `Unchanged`, `Exists`, `Rejected`, `Failed{fault}`, `FailedWithOutput{fault, diagnostic: PushDiagnostic}`.
  - There is no `Done::Cancelled`. A cancel ends as `Failed`/`FailedWithOutput` with `fault: Cancelled`, which the hold handles (hold.rs:883).
- **`Fault`** (:136) is `Missing{missing}`, `Refused`, `Unreachable`, `Broken`, `TimedOut`, `Cancelled`. `Missing` (:154) is `Repository`, `Branch`, `Commit`.
- **`Commit([u8;32])`** (:31) has `new` and `raw`.
- **`PushDiagnostic{output:[u8;512], length:u16, cut:u64}`** (:201): `CAPACITY=512` (:209), `new(&[u8], cut)` (:213, keeps the tail and adds what it dropped to `cut`), `empty`, `output`, `cut`.
- **Boundary** (boundary.rs):
  - `Event::Done{owner: Token, done}` (:60). `Request::Io{owner, op, deadline: Time}` (:79).
  - The checkout's cancel is `Request::Cancel{owner}` (:82); the root renames it `CancelIo` (route.rs:259).
  - The owner token is the hold's token (`io(id,…)`).
- **`Limits`** (limits.rs:13-38) has `workspaces`, `repositories`, `name_bytes`, `message_bytes`, `conflicts`, `path_bytes`, `remote_timeout`, `local_timeout`, `facts`. That covers every checkout limit git.md section 8 names.
- **Where the hold already agrees with git.md:**
  - It refuses over-limit or duplicate conflict paths as `Broken` (hold.rs:438-468).
  - It verifies a push that failed `TimedOut`, `Broken`, `Unreachable` or `Cancelled` with `Fetch{Want::Branch}` (hold.rs:968-1029).
  - A push that fails `Missing` or `Refused` becomes `Landing::Explained`.
  - A local operation's failure becomes `Failure::Transient`.
- **Where the code differs from git.md (all doc-only):**
  - `Op::Make` says "io removes whatever is there first" (git.rs:59); git.md moves the old workspace aside and sweeps it later.
  - `CheckOut` says "the git directory is left as it is" (:74); git.md's `checkout --force` clears the merge state.
  - `Merge` says "the checked-out branch" (:77); HEAD is detached.
  - `Fault::Missing` lists only Clone and Fetch (:137); git.md section 6 also lets Create and Push fail `Missing{Repository}`, which the hold already handles.
  - `Fault::Unreachable` (:142) leaves out "or failed" (a 5xx, the remote hanging up).
  - `Limits.name_bytes` still lists "identity" (limits.rs:19), which is now a `u32`.

## 2. The root's parts against channel.md section 5
- **Root boundary** (crates/temper-worker-domain/src/boundary.rs):
  - `Event::Assign{assignment: host::Assignment, items: wire::Workspace, save: Option<Box<[u8]>>}` (:61)
  - `Request::Answer{run, attempt, answer: host::Answer, work: wire::Work, preparation: Option<wire::Preparation>}` (:101-107)
  - `Request::Io{owner, op: git::Op, deadline}` (:122), `Request::CancelIo{owner}` (:125), `Event::Done{owner, done: git::Done}` (:84)
- **Down, in wire.rs — matches channel.md section 4:**
  - `Workspace{repositories}` (:14)
  - `Repository{tag:u32, name, remote, start, access, identity:u32}` (:20)
  - `Start{Merge{branch, base:[u8;32]}, Base{branch}, Branch{branch}, Commit{commit}, Saved{branch}}` (:41), in channel.md's order
  - `Access{Writable{push, expected: Option<[u8;32]>}, ReadOnly}` (:67)
- **Up, in wire.rs:**
  - `Work{landed: Box<[Landed]>, saved: Option<Box<[Landing]>>}` (:128). `Landed{tag, commit}` (:136) is built, filled by tag (workspace.rs:400-403).
  - `Landing{Conflicted{files}, Explained{failure: PushFailure}, Landed{commit}, Moved, Failed, Refused, Unchanged}` (:99) matches channel.md. The checkout's `Aborted` maps to `Failed` (translate.rs:145).
  - `PushFailure{repository: Option<u32>, reason, diagnostic}` (push.rs:74).
  - `PushReason` (push.rs:53) is `MissingRepository`, `MissingBranch`, `MissingCommit`, `Refused`, `Unreachable`, `Broken`, `TimedOut`, `Cancelled`, `Unavailable`, `Busy`, `TooLarge`, `Nothing`, `Unknown`; it matches.
  - `Preparation{Transient, Missing{repository:u32, missing}, Refused{repository:u32}}` (:144) names the repository by its place in the assignment, not its tag.
- **protocol.md's "Kept" change is partly built:**
  - Landed by tag: built.
  - Saved repositories by tag with `from_saved`: not built. `saved` is a bare list of `Landing` in assignment order (workspace.rs:417), and `Items` keeps only `tags` (:70).
  - A failed preparation by tag: not built. It is the checkout's place index, copied through (translate.rs:67-75; workspace.rs:349-360).
  - The root still emits `Some(Transient)` (workspace.rs:168-171); channel.md has the routing drop it.
- **Where it would change:**
  - wire.rs: tags in `Preparation`, and a new `SavedRepository{tag, from_saved, landing}` for `Work.saved`.
  - workspace.rs: `Items` (:66) gains a `from_saved` flag per repository, set in `stage` (:86) from `Start::Saved` before `spec.take()` (:243). `prepared` (:349) maps place to `tags[i]`; `wrote` (:417) and `parts` (:143) build the tagged form.
  - translate.rs `failure` (:67).
  - Tests: crates/temper-worker-domain/src/tests/{fixtures.rs:144, lifecycle.rs}; tests/worker/domain/tests/whole.rs:37 (asserts `Missing{repository: 0,…}`), src/whole.rs:407, src/runtime.rs:315; tests/system/src/composed/link.rs:24-102, which keeps its own `Items=(tag, from_saved)` to make up for the gap and would drop it; composed.rs:678.
- **Identity hand-off:**
  - Every Clone, Fetch, Create, Commit and Push carries `identity: u32`. Merge and CheckOut carry none.
  - The number is copied from `wire::Repository.identity` to `checkout::Repository.identity` (translate.rs:41-52; boundary.rs:109) and into each op (hold.rs:752-786, 850, 957, 1006).
  - The root never sees a value and never consults grants. The hub passes only LLM grants to smith (agents.rs:60-65), and the Application grants stay with the protocol layer's table.
  - The entrance check (assignment.rs:13-70) does not check that each repository's identity has an Application grant.

## 3. Smith payloads by hand-out (jig hosts.md 6.8)
- **Requests:** `Decode{agent, run, start: Box<host::Start>}` (:116), `Encode{owner, value: Encodable{Turn, CallName, Result}}` (:118/:130), `Fact{owner, body}` (:120).
- **Events:** `Decoded{agent, start: Result<Box<agent::Start>, Invalid>}` (:78), `Encoded{owner, encoded: Encoded{Turn(host::Turn), CallName, Result, Undecodable}}` (:80/:138), `Activity{owner, activity: Option<host::Activity>}` (:82).
- The hand-outs waiting to come back are kept in order in `hand_out::Pending{Turn, Call, Result}` (hand_out.rs).
- **The world plays them** in tests/worker/domain/src/protocol.rs:
  - `decode`, `encode` (:178) and `activity` (:219, with `smith_channel::Fact::decode`), using smith's codecs and keeping no state.
  - Answers are queued in order through an `owed` queue (whole.rs:161-170; runtime.rs:277-286).
  - `Io` goes to the fake checkout through `temper_worker_checkout_world::translate`.
  - Both worlds panic on `Request::CancelIo` as unscripted (whole.rs:172; runtime.rs:292).

## 4. Still done today
- **Creating a missing base branch:** yes.
  - A fetch that fails `Missing{Branch}` for a writable `Base` start (`creates_base`, hold.rs:1234) goes to `Fetch{Want::Default}` (:544-549, :774), then `Create` (:562, :776-786).
  - `Exists` leads to a re-fetch (:576); a re-fetch that finds it `Missing` is transient (:590).
  - The docs say so too (boundary.rs:121-124, wire.rs:46), and the checkout world asserts "a base branch only created".
  - `Want::Default` and `Done::Exists` exist only for this, yet git.md 4.4 still specifies `Create`.
- **Push with no expected head:** yes, in practice always.
  - A save always passes `expected: None` (hold.rs:953-956).
  - A push uses the spec's `expected`, which the engine never sets (channel.md section 4; system link.rs:237).
  - Only the checkout world exercises an expected head.

## 5. The worlds
- **Sizes:**
  - `tests/worker/checkout`: 3,393 lines (world.rs 1,291, forge.rs 382, merges.rs 240, translate.rs 191, client.rs 156; tests 1,017).
  - `tests/worker/domain`: 1,828 lines.
  - `tests/fake-checkout` (`temper-fake-checkout`): 21 lines, only re-exports of `skein-fake-checkout` (that crate is 1,848 lines at the pin).
  - `tests/system`: 3,438 lines.
- All of them use git at the domain face: `skein_fake_checkout::git::{clone_repository, fetch, create, check_out, merge, commit, commit_merging, push, push_expected}`, with the world's `Remote` standing in for the forge.
- No temper crate uses `skein-sim` or `skein-fake-machine`.
- **Does `skein-fake-machine` exist at the pin?** Yes, but its programs are a closed match in `program_of` (face.rs:76-105: echo, exit, fork-exit, fork-live, never). They produce `skein_sim::Program{Echo, Exit, Never, Fork, Service}` (machine.rs:121-132), which is also closed.
  - skein's testing.md says "skein ships no fake machine… a service's is the service's".
  - So a temper crate cannot add a program to it.
- **The extension path at the pin is a hosted program:**
  - skein-world's `HostedProgram<P>{program, make: fn(&Spawn,&Inherited)->P, instances, operations}` (program.rs:26), registered with `World::host` (world.rs:239).
  - The simulator selects it as `Program::Service` (`Sim::hosted_spawn` processes.rs:107, `bind_service` :120, `finish_service` :152).
  - It receives the exact args, env, dir and pipes (`Inherited` :14).
  - Calls that match no hosted program go to a world `Machine` trait (:90, `World::with_machine` :210), which can wrap `skein_fake_machine::Machine` for files.
  - So `temper-fake-git` would be a hosted service doing its own io, not a program added to skein's fake machine.
- **Reusable for `tests/worker/protocol-git`:** there is no scripted-io kit. `skein_world::domain::{Schedule, Stage, Ledger, Referee, Trace, assert_replays}` plus heap counting fit "scripted operations above, scripted io below". `skein_world::files::Driver` wraps a `FileIo`.

## 6. skein-io and skein-lib at the pin
- **Processes** (records.rs):
  - Requests: `Spawn{owner, spawn: kernel::Spawn}` (:43), `Signal{child, signal, to: Target}` (:45), `Stream{stream, down: stream::Down{Demand{read, room}, Send(Box<[u8]>), Finish}}` (:32), `Close{entity}` (:58, kills the group then reaps), `Abort{entity}` (:62).
  - Events: `Spawned{owner, child, pipes: Box<[Token]>}` (:106), `Exited{owner, exit}` (:111), `Stream{owner, up: Up{Bytes, Room, End, Failed}}`, `Failed`, `Closed{owner}` (:143).
- **Kernel types** (kernel.rs):
  - `Spawn{program, args, env, root: Fd, dir, pipes: Box<[Pipe]>}` (:523). `args` excludes argv[0], which is the program path. `env` is whole, as `NAME=value` entries (:520-521). `dir` resolves beneath `root`. Each child leads its own group (io.md section 6).
  - `Pipe{child:u32, way: Way{In, Out}, parent: Option<Fd>}` (:534), `Signal{Terminate, Kill}` (:550), `Target{Child, Group}` (:559), `Exit{Code(u8), Signal(u32)}` (:600).
- **File requests** (file.rs, driven by `file_layer::FileIo`):
  - `MakeDirectory{owner, root: Token, path, mode}` (:33). It splits the path into parent and leaf, and refuses links in the parent only when the root is a private (`OpenPrivate`) root (private.rs:146-152).
  - `Scan{owner, root: Token, path, max, max_bytes, no_follow}` (:97) answers `Scanned{entries: Box<[Entry{name, kind}]>, more: u64}` (:213), the first entries in name order.
  - `Rename{owner, root: Fd, from, to}` (:150): both must be single names in one directory (file_layer.rs:576-583), even though the kernel's `Op::Rename` has separate source and target directories.
  - `Remove{owner, root: Fd, name, directory: bool}` (:157), single name. `List{owner, root: Fd}` (:164).
  - `OpenDirectory{root: Fd, name, no_follow}` (:82) and `OpenReadNoFollow{root: Fd, name}` (:64) answer `Opened{file: Token}`; then `ReadAt{file, offset, max}` (:126) and `Close` (:142).
  - Events: `Renamed`, `Removed`, `Made`, `Failed{error: kernel::Error,…}`, `Cancelled`.
- **`FileIo` driver** (file_layer.rs):
  - It runs one request at a time (`takes()` :338).
  - `adopt_root(fd) -> Option<Token>` (:349); `descriptor(token) -> Option<Fd>` (:355) is a method on the driver, not a record; `down_until(deadline)` (:428), `cancel` (:648).
- **skein-lib:** there is no `ByteRing` and no capped buffer that grows by doubling. `Intake` is allocated once at its cap and refuses past it; `Writer` is exact-length. Both would be modules of the component, as performance.md section 3 already allows.

## Findings
1. Saved work up is untagged and has no `from_saved` (wire.rs:128-131; workspace.rs:417). The system world compensates statefully (link.rs:24-102). **Changes a Shape.**
2. `wire::Preparation` names the repository by place, not tag (wire.rs:144-156; translate.rs:67-75). **Changes a Shape.**
3. io's `Rename` cannot cross directories (file.rs:150; file_layer.rs:576-583). git.md 4.1's `workspaces/<t>` → `trash/<n>`, the start's move of `workspaces` and `home` into `trash`, and the sweep's child rename are all cross-directory. It needs a skein change or a different layout. **Blocks a step.**
4. `Remove`, `Rename`, `List`, `OpenDirectory` and `OpenRead` take a kernel `Fd`, but a directory opened while running comes back as a `Token`, and only `FileIo::descriptor` turns it into an `Fd`. So the sweep cannot remove `trash/<n>/<child>` through records alone. **Blocks a step.**
5. `FileIo` runs one request at a time (file_layer.rs:338). Make, the sweep and every in-flight Commit's scan share one driver, and the design does not say which driver the component owns. `MakeDirectory` refuses links only beneath a private root. **Changes a Shape.**
6. `skein-fake-machine`'s programs are closed (face.rs:76-105; sim machine.rs:121-132). The fake git program has to be a skein-world `HostedProgram`/`Program::Service` plus temper's own `Machine`, not a program "on skein's fake machine" as testing.md 4.3 and protocol.md section 5 say. **Changes a Shape.**
7. Base-branch creation is still built (hold.rs:544-586, 774-786, 1234), and git.md 4.4 still specifies `Create`, `Want::Default` and `Exists`. This contradicts worker.md section 11 "Goes". **Cosmetic** (report only, per STATUS).
8. No push in the system carries an expected head (hold.rs:953-956; channel.md section 4). This is worker.md section 13 as stated. **Cosmetic.**
9. The root doesn't check that a repository's identity has an Application grant (assignment.rs:13-70). A missing value fails `Broken`, which becomes `Failure::Transient` (hold.rs:1122-1127) and is retried indefinitely. **Cosmetic.**
10. There is no `Done::Cancelled`; a cancel is `Failed`/`FailedWithOutput{Cancelled}`. git.md section 2's wording reads as if it were a terminal of its own. **Cosmetic.**
11. `PushDiagnostic` is defined twice, identically (checkout git.rs:200-244 and worker push.rs:5-49). channel.md calls the wire's diagnostic "the checkout's". **Cosmetic.**
12. Stale doc comments in git.rs and limits.rs (`Make`, `CheckOut`, `Merge`, `Fault::Missing`, `Fault::Unreachable`, `name_bytes`), listed in part 1. Also, io puts the absolute program path in argv[0], where git.md section 3 shows `git`. **Cosmetic.**
13. The worker-domain world never plays `Request::CancelIo`; it panics on it (whole.rs:172; runtime.rs:292), so the root's cancel routing is untested there. **Cosmetic** (test gap).
