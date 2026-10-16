# Findings: what skein gives the forge-setup hosts (setup.md 6.3, 6.4, 7, 11)
At skein `main` ec686b8 (2026-10-16). Paths are skein's unless prefixed `T:` (temper's); all checked in the source.
**Absent:** 3 `execve`; 4 the lock held across exec. Present: 1, 2, 5, 6, 7, 8. No skein change is needed if the
designer accepts the std-only route of 3 and 4 (a reading of the rules, not a skein fact).

## 1. Spawn, streams, group, kill on close, exit: PRESENT
- `io::Request::Spawn { owner, spawn }` (skein-io records.rs:43); `kernel::Spawn { program, args, env, root: Fd, dir,
  pipes }` (kernel.rs:523); `Pipe { child, way: In|Out, parent }` (534). Design: io.md 6.
- `env` (`NAME=value`, kernel.rs:1112) is the child's whole environment: the `envp` of `pidfd_spawn` (skein-shell
  ring/process.rs:179-183, 265-273). `program` is a path as given, no PATH search. `root` is an open directory
  descriptor (`skein_shell::open_root`, ring.rs:1595); `dir` resolves beneath it (RESOLVE_BENEATH, process.rs:101-113).
- stdout/stderr: a `Pipe` per child fd (1, 2, `Way::Out`); `Event::Spawned { owner, child, pipes }` (records.rs:106)
  gives the parent ends as stream tokens in request order (io process.rs:115-127). Unrequested fds 0-2 are /dev/null,
  all others closed (ring/process.rs:146-160, 227). The box's own stdout: `Io::adopt_write_pipe(Fd::new(1), ..)`
  (layer.rs:167); the `drain` hook may write one short line only (shell.md 12).
- Own process group: `setpgroup(0)` (ring/process.rs:245), kept across exec. `Request::Close { entity: child }`
  SIGKILLs the group, then reaps (io process.rs:253-304); one terminal `Closed`. `Event::Exited { exit: Exit::Code(u8)
  | Exit::Signal(u32) }` (records.rs:111; kernel.rs:600) comes from `waitid WNOWAIT` (process.rs:158-201), before reaping.

## 2. Signals: PRESENT, narrow
- To a child: `Request::Signal { child, signal: Terminate | Kill, to: Child | Group }` (records.rs:45; kernel.rs:550,
  559) by `pidfd_send_signal` (ring/process.rs:290-308). Only SIGTERM and SIGKILL: no SIGINT, SIGHUP.
- One signal in flight per child; one sent during another, or once closing, is dropped silently (io process.rs:237-251).
  So the "second signal" is a second `Terminate`, sent after the first was reaped (`box.grace_ms` spaces it);
  protocol.md 158-159 has `temper serve` turn its drain into an abort at the second termination signal.
- Own signals: `Event::Shutdown { signal: ServiceSignal::Interrupt | Terminate }` (records.rs:128; kernel.rs:591) from a
  signalfd: `skein_shell::open_termination_signals()` then `Io::adopt_signals(fd)` (layer.rs:198); T: crates/temper/
  src/lib.rs:381-395 does so, before any thread starts.

## 3. `execve`: ABSENT
- No exec in skein, smith, jig or temper code (grep `execve`, `CommandExt`, `.exec(`); skein spawns by `pidfd_spawn`
  (io.md 6). `std::process::Command` is only in temper tests and tools (T: crates/temper/tests/support/live.rs:140).
- Rules. programming-model.md 2.1: no std handle types that close on drop (`OwnedFd`, `File`), "`std::process` is not
  used", "the shell's clippy.toml bans them all" (skein-shell/clippy.toml:9-15, skein's own); `unsafe` only in the ring
  adapter in code a service runs (2.1, 10.2); a shell may depend on `libc` (section 4). temper: `unsafe_code =
  "forbid"` in every crate (T: crates/temper/Cargo.toml); its clippy.toml bans only HashMap, HashSet, Instant,
  SystemTime, `thread::spawn`.
- Answer: `CommandExt::exec` is safe Rust (only `pre_exec` is unsafe), so no lint and no `unsafe` rule stops
  temper-forge-setup-shell calling it; a libc `execve` needs `unsafe`, forbidden. But 2.1's prose says the shell does
  not use `std::process`, and temper's shells follow it: the plan must state the exception (setup.md 6.4/11, the
  designer's call), or ask skein-shell for an `exec` helper (its ring adapter may use `unsafe`). Verified (scratch
  program, rustc 1.95): std's `exec()` clears the signal mask and resets SIGPIPE; `env_clear().envs(..)` sets the env.

## 4. Exclusive lock for the process's life and across exec: ABSENT
- skein's `Lock` is designed (io.md 5; kernel.md 312-324), unbuilt: no `flock` in the crates; STATUS.md:35-36 give it
  to hearth's M1 plan, not yet written. Built, it would not serve: every descriptor is close-on-exec and the lock "never
  passes to a child" (kernel.md 321), so exec ends it; setup.md 6.4 needs it held across the exec.
- Std route, verified (scratch program, rustc 1.95, `flock -n` from outside): `File::try_lock()` (std, stable since
  1.89; rust-version here is 1.95), then `Command::new(bin).env_clear().envs(..).args(..).stdin(Stdio::from(file))
  .exec()`: the exec'd process holds it as fd 0 until it exits; without the hand-over, O_CLOEXEC frees it at exec.
  Safe Rust. Caveats: Forgejo's fd 0 is the lock file, and a descendant keeping it (git, a hook) outlives Forgejo
  holding the lock; `File` is banned in skein-shell's clippy.toml, not temper's. `prepare` needs only `try_lock()`.

## 5. Private file, 0o700 directory, private-and-own check: PRESENT
- `file::Request::Store { owner, root: Token, path, bytes, expected, no_follow }` (file.rs:107): temporary
  `<name>.skein-<hex>` made 0o600 in a root opened by `OpenPrivate` (file_layer/store.rs:208), written, synced, renamed
  over, directory synced (io.md 5.2-5.3). Any other root: the old file's mode, or 0o666 less umask (store.rs:208, 766).
- 0o700 directory: `MakeDirectory { owner, root, path, mode }` (file.rs:32); at startup `skein_shell::make_root(path,
  0o700)` (ring/roots.rs:11). The check: `OpenPrivate { owner, root, path }` (file.rs:40; file_layer/private.rs) refuses
  a link, group/other bits or another owner; needs `FileIo::with_effective_user(effective_user())` (file_layer.rs:331;
  ring.rs:1666) and `adopt_root` (file_layer.rs:349); FileIo runs in a `Kernel` loop.
- temper's own check, std only: `private_root(path, user)` T: crates/temper/src/lib.rs:302-309 (`symlink_metadata`, a
  directory, not a link, `uid() == user`, `mode() & 0o077 == 0`), called at 323 and 328 around `skein_shell::make_root(
  &root_path, 0o700)` (325); `user` from `effective_user()` (317). Reads: `read_private_file` (startup.rs:43; T: lib.rs:132).

## 6. Timers and a step machine hosted on skein-io: PRESENT
- Each layer owns `skein_lib::Deadlines` (programming-model.md 9); the host reports `Host::next_deadline`
  (skein-shell drive.rs:27-32); `drive` waits on one ring timeout for it (drive.rs:65-73).
- Host example: T: crates/temper/src/host.rs:18-90 (`impl skein_shell::Host`: `iterate`, `completions`, `submissions`,
  `work_pending`, `next_deadline`, `next_policy_deadline`, `is_empty`, `exit`, `drain`), started at T: lib.rs:381-395.
  Prior art with processes and timers: smith-protocol-machine/src/process.rs (Terminate 262-285, Kill 349-374, 392).

## 7. skein-http: PRESENT (basic auth by hand)
- Client: `skein_http::client` (client.rs:157: `Call`, `Upload`, `Body`, `Discard`, `Close`) over a `lib::stream`, a
  socket or skein-tls's plaintext. T: crates/temper-engine-protocol-forge/src/connection.rs:121-133 (`Transport::
  Plaintext => None` omits TLS; settings.rs:39-43), head at 160-172 (`Authorization: token <t>`), socket by
  `io::Request::Connect`. Plaintext is loopback only (limits.rs:112).
- Basic auth: not in skein-http; the caller sets the header with `skein_lib::base64::{encoded_len, encode}`; T: crates/
  temper-worker-protocol-git/src/invocation.rs:181-187 builds `Authorization: Basic ..` so. Not a skein change.
- Server: `skein_http::server` (http.md 5): `Next` then `Call`; the head goes down when the layer above sends
  `Respond` (5.4), so an answer can wait on other work. Used by jig-protocol-engine-clients/src/connection.rs:4,82;
  temper has none in production yet (only the test crate temper-fake-forge-protocol; no webhook listener). Two
  listeners: yes, each an io entity; T: crates/temper-service/src/workers.rs:60-71 issues a `Request::Listen` per address.

## 8. "skein-shell" and skein-tls
- skein-shell (shell.md; lib.rs:1-22) is not a command shell: it is what a service's `main` runs its loop on: `Kernel`
  (io_uring), `Clock`, `seed`, `Host` + `drive`, startup helpers (`open_root`, `make_root`, `effective_user`,
  `read_private_file`); its ring adapter holds the only `unsafe` a service runs. Linux only. Here: the loop and checks.
- skein-tls (tls.md 1): TLS client over rustls, no server side. Here: reconcile against a connected or managed forge
  over TLS (`Transport::Tls`); an owned forge is plaintext on loopback (setup.md 62-65, 430).
