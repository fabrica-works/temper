# Session 09: the real loop's stories over more than one call

temper 08's world, with the stories of more than one call or run
(tools.md, 11.3; testing.md, 2.4): a restart keeps the work (09.1); a
lost answer reached under its name (09.2); DIRECTORY's lock (09.3); a
record the tools cannot read (09.4). It extends 08's crate, fixtures and
steps by name, never redefines them.

Needs at start: temper 08's world (08.1 to 08.4). Each increment names
what it needs of temper 05's acts, waits, `pending` and lock, and of the
tools plan's temper-03.4, the kept lines; temper 05's names below are
those `05-ctl.md` sketches, created by temper 05 (`host::OneShot`, `limits`): check each as
landed.
Steps: `temper-09.1`, `temper-09.2`, `temper-09.3`, `temper-09.4`.

## Read first

- temper's `docs/design/client/tools.md`: 5.4, 5.6, 6.1, 6.3, 7, 8.1 to 8.3, 9,
  11.3, 12; `docs/design/domain/testing.md`: 2.4, 5.2, 6.1.
- jig's `docs/design/protocol/client.md`: 8 (the printed link), 10 (the
  small store); `client/domain/client.md`: 11; `client/domain/
  contract.md`: 3.1. skein's `io.md`, 5 (`Lock`); `kernel.md`, 6.2.
- This plan's `decisions.md` (design questions 1 and 3; 4, 5, 8),
  `temper/08-real-loop.md` whole, `temper/10-process.md` (10.2, the
  terminated call).
- The code: skein-world's `real` (`kill`, `settle_children`,
  `spawn_with_fds`), skein's `tests/world/tests/real.rs`
  (`kill_drops_child_and_settles_pending_read_before_parent_wait`);
  `tests/one-process/src/referee/durable.rs` (`Capture::complete`,
  `is_log`); jig's `crates/jig-protocol-store/src/keys/people.rs`
  (`Key::Answer`); `crates/temper-client-tools/src/key.rs` (`engine`)
  and `guide.rs`; `jig-client-shell-native`'s `Shell`;
  `jig-client-protocol-link`'s `store.rs` (`start_store`) and its
  fixture `other_store_version` (`src/tests/fixtures/link_store.rs`).

## What the design asks

- **11.3, a restart retains the work:** 09.1.
- **11.3 and 7, a lost answer made once,** from another DIRECTORY signed
  in as the same party, with one chat: 09.2. The same party is the
  printed link again: it signs in as the first owner it names, and a
  refused sign-in draws a fresh one (jig's `protocol/client.md`, 8;
  `crates/temper-service/src/clients.rs`).
- **8.2 and 6.1, the next call that sends** sends what DIRECTORY kept
  first; its line names operation and outcome, no name, no key. 09.2.
- **8.3, the lock:** acts in turn; none for reads, waits and `pending`;
  it ends with the process, however it ends. 09.3.
- **5.6 and 11.3, an unread record:** `pending` counts it; the next call
  that sends says `dropped:`. 09.4.
- **8.2 and design question 3, a signal:** a killed call's status is the
  OS's, `Exit::Signal(9)`: SIGKILL reaches no host. `Shutdown`'s 128 + N
  is temper 10.2's (143).
- **11.3, another party's kept request,** and **5.4, a wait across a
  restart:** not built (Left for later).
- **11.3 and testing.md 5.2, a failure kept with its trace:** asked of
  the tier, not of a story; nothing here.

## Choices this plan makes

- **The kill lands before the answer, by the order.** 08's order is the
  peer, `ServiceHost`, the caller, then the calls. `ServiceHost` sees
  the engine's durable hold in its turn; the engine's answer, submitted
  in that turn, reaches the kernel at the pass's `ring.enter`; the
  caller's `Op::Signal { signal: Signal::Kill, .. }`, in the same pass,
  ends the call before its next turn, so its host never reads the answer.
- **Kill, not Terminate:** `Signal::Terminate` is the host's `Shutdown`,
  exit 143 (design question 3; temper 10.2); a dropped root gets no turn.
- **The durable cue is `ServiceHost`'s.** Armed with a name's engine key
  (`temper_client_tools::key::engine`), it follows `Capture::complete`:
  opens whose path `is_log` names, each completed write searched for the
  16 key bytes with a 15-byte carry, then a successful `Sync` of that
  descriptor. The people store's `Key::Answer` holds the key raw, as
  skein-kv's frames (`crates/skein-kv/src/frame.rs`) write keys.
- **The second DIRECTORY signs in by the link, never a copied token.**
  `same_party(D2)` signs in with `<scratch>/link`, which D1 spent, is
  refused, waits for `ServiceHost`'s next link, and signs in again: one
  `Called`, `tries` 2, the refusal's status unasserted.
- **The lock's holder is a probe:** the caller's own `Op::Lock` on
  DIRECTORY until a `release` step, the hold a sending call takes, so a
  call meets it by construction, never by timing.
- **The unread record is written by jig's own store writer:**
  `jig_client_shell_native::Shell`, saving a record built from jig's
  wire types as `other_store_version` builds it.
- **A restart is two runs on one scratch:** `Ran::into_scratch`, then
  `run_on`.

## Rules that bind it

- **08's rules hold:** nothing upstream, nothing of `tests/one-process`;
  limits from the builders; no sleeps for order; a kill only at a cue
  (`held`, `locked`).
- **One behaviour per test,** no "and" in a name (the plan's README); a
  helper two stories need is the world's.

## Increments

### 09.1 A restarted service keeps the chat with its reply

Scope: a chat to its reply, the service drained, a second service on the
same scratch, then the chat read (tools.md, 11.3; testing.md, 2.4).
Needs: temper 05's reads, `sign-in` and `chat` act, as 08.3.
Files: `tests/real-loop/src/{lib.rs, calls.rs, fixtures.rs, world.rs,
service.rs}`, `tests/real-loop/tests/restart.rs` (new).
Shape:
- `fixtures::write` on a kept scratch rewrites the documents and the
  handed-in account (the new peer's and forge's addresses), and keeps
  the store, the parents and each DIRECTORY.
- `ServiceHost` replaces `<scratch>/link` with each link it prints and
  counts them (`ServiceHost::links`); the referee releases the caller
  once this run's `ServiceHost` has printed one, not once the file
  exists.
- `Ran::number(call)`: the task number call's `done: task N` printed.
  `Arg::Number(n)`, `calls::number(n)`, which `argv!` takes as `task(..)`.
Test shape: `tests/restart.rs`, the counting allocator, as 08.2's.
```rust
#[test]
fn a_restarted_service_keeps_the_chat_with_its_reply() {
    use temper_real_loop_world::{argv, calls::{call, number, occurrences, sign_in, task, until, D1}, llm, run, run_on, scripts, Exit};
    let first = run(|| scripts::chat_scripts(1), || vec![
        sign_in(D1),
        call(D1, argv!["chat", "--project", "1", "--key", "kept", scripts::WORDS]),
        until(D1, argv!["chat", task(1)], scripts::REPLY, scripts::EARLY),
    ]);
    let chat = first.number(1);
    let second = run_on(first.into_scratch(), || llm::scripted(scripts::chat_scripts(1)),
        move || vec![sign_in(D1), call(D1, argv!["chat", number(chat)])]);
    let read = &second.calls()[1];
    assert_eq!((read.exit, occurrences(&read.stdout, scripts::REPLY)), (Some(Exit::Code(0)), 1), "{}", String::from_utf8_lossy(&read.stderr));
    let asked = second.llm().iter().any(|seen| matches!(seen, skein_fake_peers::llm::Observation::Query { .. }));
    assert!(!asked, "the restarted service asked the model again");
}
```
Cost: two preparations, the store reopened at the second, one agent turn
and two drains: about twice 08.3's story, the likeliest past 1 s. Measured
as 08.2's (median of three in the commit body); past 1 s it is profiled,
then moved whole to `tests/fuzzy_restart.rs` (~5 s), never merged.
Proof: joint (the real service, peer and kernel), no higher than its
test: the layer above, the executables, is temper 10's process tier.
Don't: name a port for the second service; copy run 1's token.
Size: at most ~90 lines of `tests/real-loop/src`, counted as 08.1 (`git
diff -M --numstat origin/main...HEAD -- tests/real-loop/src`, summed by
awk); 0 by part 1's count.
Removes: nothing.

### 09.2 A lost answer is reached under its name

Scope: a chat killed once the engine holds it; its name asked again from
D2, signed in as the same party; then from D1 by the next act (tools.md,
7, 8.2, 11.3).
Needs: temper 05's `chat` act with the kept requests sent first (8.2,
step 4); the tools plan's temper-03.4, the kept lines.
Files: `tests/real-loop/src/{calls.rs, service.rs, world.rs, lib.rs,
fixtures.rs}`, `tests/real-loop/tests/lost.rs` (new), `tests/real-loop/Cargo.toml`
(`temper-client-tools`), root `Cargo.lock` (hotspot: forge kit).
D2's parent is `fixtures::write`'s, as D1's: if 08.2's `write` makes
D1's alone, 09.2 makes D2's there (ruling 106).
Shape:
- `service::Cue`, after `Capture::complete` (Choices). The referee arms
  `ServiceHost` with each `held` step's key before the run starts.
- `held(name)` holds the caller until the cue fires. `kill(call)`: the
  caller's `Op::Signal` with `Signal::Kill` on call's pidfd, then its
  `Op::Wait`; kept at call's index, `exit` the wait's.
- `same_party(at)` (Choices): `Step::SameParty`, a call at its index.
- `run`'s end check: as many killed as `kill` steps, each
  `Exit::Signal(9)`, in place of 08's "nothing killed".
- `calls::kept(call)`, an `Arg`: the task number of the one `kept:` line
  on call's stderr. `calls::listed(stdout)`: the numbers that begin a
  `tasks` page's rows (`N · ...`, `temper-client-tools`'s
  `src/tests/golden/tasks_next.txt`).
  `calls::kept_lines(stderr)`; `calls::nameless(stderr, name)`: neither
  the name nor its engine key, raw or in hex, on stderr.
Test shape: `tests/lost.rs`, the counting allocator.
```rust
#[test]
fn a_lost_answer_reaches_its_name_from_another_directory() {
    use temper_real_loop_world::{argv, calls::{call, held, kill, listed, same_party, sign_in, start, task, until, D1, D2}, run, scripts, Exit};
    let ran = run(|| scripts::chat_scripts(1), || vec![
        sign_in(D1),
        start(D1, argv!["chat", "--project", "1", "--key", "lost-answer", scripts::WORDS]),
        held(b"lost-answer"),
        kill(1),
        same_party(D2),
        call(D2, argv!["chat", "--project", "1", "--key", "lost-answer", scripts::WORDS]),
        until(D2, argv!["chat", task(3)], scripts::REPLY, scripts::EARLY),
        call(D2, argv!["tasks", "--project", "1", "--label", "chat"]),
    ]);
    let [_, lost, _, again, _, list] = ran.calls() else { panic!("six calls") };
    assert_eq!((lost.exit, lost.stdout.as_slice()), (Some(Exit::Signal(9)), b"".as_slice()), "killed before its answer");
    assert_eq!(again.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&again.stderr));
    assert_eq!(listed(&list.stdout), [ran.number(3)], "the party has one chat");
}
```
`a_killed_calls_request_is_answered_by_the_next_act_on_its_directory`:
`chat_scripts(2)`; sign-in, the start, `held`, `kill(1)`, then `chat
--key next-act` on D1 and `until` each chat's reply (`kept(2)`,
`task(2)`); the act exits 0, its stderr has one kept line, beginning
`kept: chat, done: task `, and `nameless(.., b"lost-answer")`.
Cost: one or two agent turns, a kill, a link drawn again: each about
08.3's, under 1 s; measured as 08.2's.
Proof: joint, as 09.1's.
Don't: kill on a time or on the call's output; read D1's token for D2.
Size: at most ~220 lines of `tests/real-loop/src`, counted as 09.1.
Removes: 08.2's "nothing killed" end check, replaced by the count of
`kill` steps.

### 09.3 DIRECTORY's lock

Scope: two acts take the lock in turn; a read, a wait and `pending` run
while it is held; a killed holder's lock is free to the next act
(tools.md, 8.3).
Needs: temper 05's lock (its retry pace, its `limits`), its reads,
waits and `pending`.
Files: `tests/real-loop/src/{calls.rs, world.rs, lib.rs}`,
`tests/real-loop/tests/lock.rs` (new).
Shape:
- The caller gets a root on the scratch (`World::spawn_with_fds`).
  `probe(at)`: `Op::Open { how: OpenHow::Directory, .. }` of DIRECTORY,
  then `Op::Lock`, which must complete taken; `release(at)`: its
  `Op::Close`.
- `fixtures::CTL`'s factory wraps temper 05's `OneShot` in
  `calls::Recorded`, which notes each `Op::Lock` the host submits and its
  completion before forwarding it, as `ServiceHost` observes the
  service's: `Called.locked`, when one completed taken, and
  `Called.busy`, how many completed `Busy`.
- `contended(calls)` holds the caller until each named call's `busy` is
  at least 1; `locked(call)` until call's `locked` is set.
Test shape: `tests/lock.rs`, the counting allocator.
```rust
#[test]
fn two_acts_on_one_directory_take_its_lock_in_turn() {
    use temper_real_loop_world::{argv, calls::{contended, join, probe, release, sign_in, start, task, until, D1}, run, scripts, Exit};
    let ran = run(|| scripts::chat_scripts(2), || vec![
        sign_in(D1),
        probe(D1),
        start(D1, argv!["chat", "--project", "1", "--key", "first", scripts::WORDS]),
        start(D1, argv!["chat", "--project", "1", "--key", "second", scripts::WORDS]),
        contended(&[1, 2]),
        release(D1),
        join(1),
        join(2),
        until(D1, argv!["chat", task(1)], scripts::REPLY, scripts::EARLY),
        until(D1, argv!["chat", task(2)], scripts::REPLY, scripts::EARLY),
    ]);
    let (one, two) = (&ran.calls()[1], &ran.calls()[2]);
    assert_eq!([one.exit, two.exit], [Some(Exit::Code(0)); 2], "{}", String::from_utf8_lossy(&two.stderr));
    let (a, b) = (one.locked.expect("one took the lock"), two.locked.expect("two took the lock"));
    assert!(one.ended <= b || two.ended <= a, "both held DIRECTORY's lock at once");
}
```
`calls_that_send_nothing_run_while_the_lock_is_held`: a chat to its
reply, `probe(D1)`, then `task N`, `pending` and `wait task N --until
replied`, then `release(D1)`; each exits 0 with `locked` `None` and
`busy` 0.
`a_killed_holders_lock_passes_to_the_next_act`: a started chat,
`locked(1)`, `kill(1)`, then `chat --key next-act` and its reply; the act
exits 0 with `busy` 0. Killed one turn after its lock, the holder has
kept nothing, so `chat_scripts(1)` suffices.
Cost: one or two agent turns; the first story waits on at least one
100 ms retry: each about 08.3's, under 1 s; measured as 08.2's.
Proof: joint, as 09.1's: `flock` on the real kernel, which no lower
world has.
Don't: retry a lock in the caller; hold the probe past the story's steps.
Size: at most ~160 lines of `tests/real-loop/src`, counted as 09.1.
Removes: nothing.

### 09.4 A record these tools cannot read

Scope: DIRECTORY's store holds a record of a store version the tools do
not read: `pending` counts it; the next call that sends says it dropped
it (tools.md, 5.6, 6.1, 11.3).
Needs: temper 05's `pending` and its store's load; the tools plan's
temper-03.4, the `dropped:` line.
Files: `tests/real-loop/src/{unread.rs (new), fixtures.rs, world.rs,
calls.rs, lib.rs}`, `tests/real-loop/tests/unread.rs` (new),
`tests/real-loop/Cargo.toml` (`jig-client`, `jig-client-shell-native`,
`jig-client-protocol-wire`; `temper-client-tools` comes with 09.2,
`skein-io` and `skein-lib` with 08.2), root `Cargo.lock`.
Shape:
- `unread::record()`: a `jig_client::Record` keyed by
  `temper_client_tools::key::engine` of the
  `temper_client_tools::jig::key::Name` `unread`
  (its `new` is an `Option`), whose bytes are
  `jig_client_protocol_wire::Kept::new(&CEILINGS, KeptParts {
  store_version: u16::MAX, record: Box::default() })` measured and
  encoded into a `skein_lib::Writer`, as `other_store_version` does.
  `start_store` drops it on its version before reading the rest.
- `fixtures::UNREAD` hosted as `fixtures::CTL` is (`host_roots`, DIRECTORY's
  parent): `unread::Unread` drives a `Shell` (`Settings` from temper 05's
  `limits`, `engine` `127.0.0.1:0`, never dialled), `adopt`s DIRECTORY,
  and once `loaded()` answers, `down(jig_client::Request::Save { .. })`
  of the record; at `jig_client::Event::Stored` it closes and exits 0,
  at `NotStored` 1. Its `Event` match names every variant
  (`wildcard_enum_match_arm` is denied), and `Unread`'s `Debug` is
  written by hand, since `jig_client_shell_native::Shell` has none
  (`missing_debug_implementations` is denied).
- `unread(at)`: `Step::Unread`, a call at its index.
Test shape: `tests/unread.rs`, the counting allocator.
```rust
#[test]
fn pending_counts_a_kept_record_these_tools_cannot_read() {
    use temper_real_loop_world::{argv, calls::{call, occurrences, sign_in, unread, D1}, run, scripts, Exit};
    let ran = run(|| scripts::chat_scripts(1), || vec![sign_in(D1), unread(D1), call(D1, argv!["pending"])]);
    let [_, wrote, pending] = ran.calls() else { panic!("three calls") };
    assert_eq!([wrote.exit, pending.exit], [Some(Exit::Code(0)); 2], "{}", String::from_utf8_lossy(&pending.stderr));
    let line = b"1 kept records these tools cannot read; a call that sends drops them";
    assert_eq!(occurrences(&pending.stdout, line), 1, "{}", String::from_utf8_lossy(&pending.stdout));
}
```
`a_sending_call_says_it_dropped_the_unread_record`: sign-in, `unread`,
`chat --key after-unread` and its reply; the act exits 0 and its stderr
holds `dropped: 1 kept records these tools cannot read` once.
Cost: the first has no agent turn, well under 08.3's; the second about
08.3's; measured as 08.2's.
Proof: joint, as 09.1's: the shell's store on the real filesystem, read
by the call that the tools' host runs; `pending`'s count of a decoded
store is temper 05's unit test.
Don't: write the store's bytes by hand or with another encoder; name a
port the writer dials.
Size: at most ~130 lines of `tests/real-loop/src`, counted as 09.1.
Removes: nothing.

## Interface sketches

What 09 adds to 08's (temper 13 extends both):
```rust
// tests/real-loop/src/lib.rs
impl Ran {
    /// The task number call `call`'s `done: task N` printed.
    pub fn number(&self, call: usize) -> u64;
}
pub struct Called {
    /* 08's fields, then: */
    /// When its `Lock` on DIRECTORY completed taken; `None` if it took none.
    pub locked: Option<Time>,
    /// Its `Lock` completions that were `Busy`.
    pub busy: u32,
}

// tests/real-loop/src/calls.rs; SameParty and Unread are calls, kept at their index.
pub const D2: Dir = Dir(2);
pub enum Arg { Lit(&'static str), Task(usize), Number(u64), Kept(usize) }
pub enum Step {
    /* 08's variants, then: */
    SameParty { at: Dir }, Held { name: &'static [u8] }, Kill { call: usize },
    Probe { at: Dir }, Release { at: Dir }, Contended { calls: &'static [usize] }, Locked { call: usize },
    Unread { at: Dir },
}
pub fn number(task: u64) -> Arg;
pub fn kept(call: usize) -> Arg;
pub fn same_party(at: Dir) -> Step;
pub fn held(name: &'static [u8]) -> Step;
pub fn kill(call: usize) -> Step;
pub fn probe(at: Dir) -> Step;
pub fn release(at: Dir) -> Step;
pub fn contended(calls: &'static [usize]) -> Step;
pub fn locked(call: usize) -> Step;
pub fn unread(at: Dir) -> Step;
pub fn listed(stdout: &[u8]) -> Vec<u64>;
pub fn kept_lines(stderr: &[u8]) -> Vec<&[u8]>;
pub fn nameless(stderr: &[u8], name: &'static [u8]) -> bool;
/// temper 05's `OneShot`, each `Lock` it submits and its completion noted before forwarding.
pub struct Recorded { /* the host, the lock's token, locked, busy */ }

// tests/real-loop/src/service.rs
/// The engine's durable hold of one named request, after one-process's `Capture`.
pub struct Cue { /* the key, the log's descriptors, a 15-byte carry, the write that held it */ }
impl ServiceHost {
    pub fn arm(&mut self, key: skein_lib::RequestKey);
    pub fn held(&self) -> bool;
    pub fn links(&self) -> u32;
}

// tests/real-loop/src/fixtures.rs
pub const UNREAD: &[u8] = b"/real-loop/unread-record";

// tests/real-loop/src/unread.rs
pub fn record() -> jig_client::Record;
/// Saves `record()` in DIRECTORY's small store through jig's native shell, then exits.
pub struct Unread { /* the shell, the root, saved */ }
```

## Left for later

Rows of this plan's `later.md` (this session edits none), settled by
decisions.md 9 and 10:
- **Another party's kept request** (tools.md, 11.3) comes with area 3's
  `sign-in --service FILE` (tools.md, 5.5 and 9): area 1 signs in only
  by the printed link, always as the first owner. Its lines are temper
  05's units meanwhile (decision 9).
- **A wait across a restart** (tools.md, 5.4) comes with the faults row:
  the restarted engine listens on a new port (testing.md, 2.4), so the
  story drops and restores the loopback link instead (decision 10).

## Questions to raise when reached

- **The cue never fires, or fires late** (the killed call printed
  `done:`): raise it with the writes traced; never a time.
- **The refused sign-in draws no fresh link,** or D2 lands as another
  party; **the second run refuses the kept scratch,** or asks the model
  again: raise it with what was observed.
- **A story past 1 s:** profiled, then raised with what it waits on;
  the restart first to `tests/fuzzy_restart.rs`. The stories together
  past 4 s with the other worlds below the domain (testing.md, 6.1) hold
  no story within its 1 s (README, Rules, "The worlds' sum").
- **temper 05's or temper-03.4's names or lines** differ from `05-ctl.md`'s
  and tools.md 6.1's: follow the landed code; a kept name or key printed
  is a finding. **The native shell dials at start:** raise it.

## Done when

Each prints what it says, on `origin/main`, after `git fetch`:

- `git grep -c -E 'fn (a_restarted_service_keeps_the_chat_with_its_reply|a_lost_answer_reaches_its_name_from_another_directory|a_killed_calls_request_is_answered_by_the_next_act_on_its_directory|two_acts_on_one_directory_take_its_lock_in_turn|calls_that_send_nothing_run_while_the_lock_is_held|a_killed_holders_lock_passes_to_the_next_act|pending_counts_a_kept_record_these_tools_cannot_read|a_sending_call_says_it_dropped_the_unread_record)\(' origin/main -- tests/real-loop/tests`
  prints counts that sum to 8.
- `git grep -n -F -e 'Signal::Kill' -e 'Op::Lock' -e 'jig_client_shell_native::' origin/main -- tests/real-loop/src`
  prints at least three lines.
- `git grep -n -E '127\.0\.0\.1:[1-9]' origin/main -- tests/real-loop`
  prints nothing.
