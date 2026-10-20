# M1 lab pieces

**The experiment is not yet runnable.** These are S's independent pieces from
`docs/design/qa.md` (amended design `998a5f054`), reusable by H/P. No
`experiment.json` or configuration templates with guessed members are supplied.
Python 3.11+ standard library is the authorized temporary lab implementation;
none of this is product code or a temper world/gate test.

From the repository root, after the trusted seed has signed in:

```sh
/bin/sh lab/chat-change-smoke/tools/ctl TEMPER CLIENT_DIRECTORY SERVE_LINK me
/bin/sh lab/chat-change-smoke/tools/ctl TEMPER CLIENT_DIRECTORY SERVE_LINK help task
/bin/sh lab/chat-change-smoke/smoke/story TEMPER CLIENT_DIRECTORY SERVE_LINK
```

`TEMPER` is the run's actual subject executable, `SERVE_LINK` its full printed
`http://HOST:PORT/jig/v1/sign-in/link?secret=…` URL. The three prefix arguments
are trusted runner inputs. The wrapper takes area 1's tool names, refuses
`sign-in`, `sign-out`, `--call` and `--call=…` in any argument, validates numeric
HTTP loopback and pins the directory's address to the link's exact authority
on every call (including help). The actual host's file is **one final-newline
line**, not JSON. The wrapper uses shell builtins, then execs the subject;
help omits DIRECTORY, and stdin/stdout/stderr/status belong to ctl. It reads no
token or other credential and records nothing. Directory ownership, private
modes and no-follow checks remain the actual host's too; the lab tool boundary
keeps the driver from writing this directory. Source: `crates/temper/src/ctl.rs`,
`temper-client-shell-cli/src/{arguments,directory,signing,host}.rs` under
`crates/`, and `client/tools.md`, 8.1.

The story makes exactly one `chat --project 1 --key smoke-chat` with the exact
request in qa.md, 3.3; extracts `done: task N, chat`; watches
`wait task N --until ended --timeout 110`; then reads `task N --tree`.
The runner must enforce the approved **120s total command budget**. There is no
sleep, application polling or new deadline. The private Python text reader
checks the actual task header, historical result JSON, digest, wait's `met`
and the ended tree root; failures/malformed output exit nonzero. Ended includes
Failed/Cancelled as ctl's `ended` condition does; it does not establish landing.
`checks/_end_state.py` judges landing independently. No full story success has
been exercised against a product cluster yet.

`NOTICE` is exactly `b'Maintained by temper.\n'`, 22 bytes. The private pure
`_judge` takes explicit completeness/loss knowledge, the last target
Merged/Moved counts and complete byte trees from the fake's objects. It returns
pass/fail/undecided (0/1/2), stable qa.md signatures, and deterministic evidence
(C, P, sorted paths and NOTICE bytes). No missing loss record means zero; unknown
flush, truncated lines or unreadable objects stay undecided. Its private
`_Object` follows the actual fake domain's parent/merge-parent/byte-tree model;
it is **not an observation encoding or a public normalized schema**. No
`checks/end-state` executable is supplied until its real adapters can work.

Exact checks, run from this directory's repository root, through heavy only:

```sh
heavy -- /bin/bash -c 'time python3 -B -m unittest discover -s lab/tests/unit -v'
heavy -- /bin/bash -c 'time python3 -B -m unittest discover -s lab/tests/process -v'
```

The unit tests use known binary content/trees, loss/incomplete/refused stories
and ctl output. Process tests use the real POSIX shell and `/bin/echo`,
`/bin/cat`, `/bin/true`, `/bin/false` canaries for guard/stream/status behavior,
and refusal of malformed story output. They introduce no fake product/harness.
No gate integration, cluster, model or paid calibration is run here.

Validation in the implementation sandbox: both commands above were attempted
with an outer shell `time`. Heavy exited 1 before queueing either test command:
connecting to `/run/user/1000/pueue_free.socket` was denied (EPERM), taking
0.206s for unit and 0.187s for process dispatch. **No tests started or passed;
test execution times are unknown.** `/usr/bin/time` is absent here, so the
commands use bash's builtin timing. Staging was also denied: the shared Git
worktree index is on a read-only filesystem. Files remain for native testing,
review, commit and branch push by the owner; no gate or land was attempted.

Still required to finish S's canonical document/templates and runnable checks:

- First-run's fake CLI/documents, required TLS/root, cued tool-call scripts,
  CI content cue, widget ID and seed without NOTICE; fake git by argv[0].
- Cross-process deliveries, serve's bound webhook address available to
  reconcile, listener readiness and bootstrap link interface. No null webhook
  configuration, polling variant or shortened fallback is substituted.
- Fake-owned decimal u64 seed derivation and named 0600 key/token output
  interfaces. The runner will supply seeds and paths, never credential bytes.
- First-run's actual observation encoding and complete flush reachable before
  stop with explicit loss count 0; the forge promise-check wrapper/judgment.
- The designed private git `objects` route, including real request/response
  encoding and complete C/P trees. Then add the private decoder/route adapter
  and `checks/end-state`, stderr evidence and runner check result encoding.
- Director-routed `add-key` and `wait forges --until connected:1`, plus the
  runner's ready captures, collection and command/check contracts. Then render
  `forge.json`, `llm.json`, serve/agent/charter/worker files and `experiment.json`
  against those exact landed contracts, and run S's three attended seeds.

Concrete source mismatches at this branch's base: `crates/temper/src/lib.rs`
accepts `serve CONFIG.json`, has no `fake` command, and ctl's
`crates/temper-client-tools/src/table.rs` lacks `add-key`/`forges`. qa.md, 5.1's
sign-in table prefixes `http://` to a capture described elsewhere as a full
URL; the wrapper/story expect the full printed URL without doubling it.
The fake Observation (`observe.rs`: Merged.commit, Moved.to, Refused) and Object
(`git.rs`: u64 counts, byte-path/content Tree) were source-read in skein's
`testing/skein-fake-forge-domain/src/`; the designed route is skein's
`fake-forge.md`, section 4. This is domain evidence, not a landed wire API.

Memory verification used `home-free-src-rust-temper`, generation
`2026-10-20T15:48:45Z`; relevant ctl paths matched metadata except the noted
changed docs/rendering/startup paths, which were source-read. Skein's graph
generation `2026-10-19T04:48:30Z` does not track those fake domain paths, so
their source is the evidence. No index was written or deleted.
