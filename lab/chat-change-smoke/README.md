# S assembly and integration review

This is the no-driver NOTICE smoke from `docs/design/qa.md`, sections 4–8.
It uses case-2's landed forge/reader contracts and the attended host's
private bootstrap. No successful product run is claimed. Use the
standard `lab-experiment/1` fields; `about` describes the host dependency,
never adds a credential capture or a new runtime protocol.

The intended process order is forge → TLS LLM → engine → w1/w2. Serve
arguments, memory declarations, worker registrations and QA templates derive
from `../cluster/`. Both remote workers have one slot; the engine has none.
`git` must be the subject binary materialized under basename `git`. Processes
inherit no environment; scripts use explicit `PATH=/usr/bin:/bin`.

The fake LLM writes `llm.key` and its test root `root.der` before readiness.
Engine and agents use TLS dialect 3, server name `skein.test`, trusting that
root alone. The existing cluster-first/second-chat scripts and their chat
wait/run windows remain. S's manifest runs `smoke/_runtime_llm.py`: after normal
rendering it derives mode-0600 `llm.s-runtime.json` beside the rendered input,
then execs the same fake LLM argv with only its document path replaced. Only
NOTICE root's third `finish` becomes text plus `wait`. The canonical template,
root delegation and initial reply/wait, coding/review scripts, TLS, seeds,
readiness and eight-turn budgets remain. NOTICE delegates to procedure 2, coding
charter 2, exact write/shell/change results and a review at the named head.
The landing fixture supplies grants, CI/up-to-date/exact-review rules and
capacities, without its shortened forge polls or rate window. Typed limits
and template checks are not proof of runtime admission or measured minima.

The five trusted seed actions are sign-in, add-key, account wait, forge wait,
adopt. Adoption precedes the story's first chat. `tools/ctl` and `smoke/story`
take `TEMPER CLIENT_DIRECTORY PUBLIC_ORIGIN`; the last argument is exactly
`http://HOST:PORT/`, never a sign-in link or path. The seed's sign-in command
declares no stored stdin: the host supplies the newline-terminated credential
through its private pipe.

The story creates ROOT, waits for its reply, reads its complete live tree
and ended direct children, and selects exactly one connector-1/kind-2 coding
procedure. It waits for that CHILD to end and requires the actual owner JSON
`result.ending.task.done.change`: kind 1, part connector 1, uncut whole merge
ID. Its requester must be ROOT. It retains this historical evidence even if
CHILD has left the final complete, unfolded ROOT tree, which must have no
held descendants and zero live delegates. Partial pages are rejected. No
model report or process exit alone establishes success.

One absolute 220s story deadline bounds every subprocess and both product
watches (`replied` for ROOT, `ended` for CHILD), using only time remaining.
S leaves ROOT Open: Close, Open-run completion and root finalization are
excluded coverage, owned by close-finish. No story waits for ROOT to end.
Seed waits200/story220 stay within command240;
ready120/stop20 and existing service graces and memory declarations stay.

The required evidence order is live observations with an independently
confirmed causal checkpoint boundary, then live C/P object reads and raw
request/response evidence; confirmed engine exit and empty trees; copied
`data/state/store`; then the existing `fake forge check` using that copy.
The manifest lists the commands, while the attended host owns this order.
The generic runner's collect/check-before-stop path cannot execute this S.
No checker opens a live store. A successful outer exit or EOF alone never
confirms the boundary; absent proof must pass `unknown` to the helper.

The host appends the helper's existing `--boundary confirmed|unknown`,
`--deadline-monotonic D`, `--evidence-dir E`, and `--result R` arguments to
`checks/end-state OBSERVATIONS FORGE_ADDRESS`. D is the actual absolute host
command deadline; E is a fresh existing evidence directory and R its result
path. The helper retains its own 120-second total cap. Preserve the raw
observation file, selected complete explicit zero-loss checkpoint prefix,
causal proof, C/P/tree/NOTICE stderr, object request/response bytes, result
and exit status. The promises command derives independent decisions from
temper records; there is no alternate credential or decision input schema.

Canonical integration contracts:

- **Shared documents:** S's forge, engine, agents, workers, charters, TLS
  and secret/digest templates equal `../cluster/` byte for byte. The engine
  connects to the fake's actual API/git addresses, takes over its handover
  through records-only setup, and registers the bound loopback webhook
  listener. No external reconcile or file-backed forge credentials remain.
  Workers use the subject's basename `git` artifact and root `workspaces`.
- **Process adaptation:** canonical forge seed is literal `17`, with the
  forge ready before the LLM. S draws `{seed.forge}` and `{seed.llm}`.
  Canonical engine stdout retains its in-memory `http://` link capture for
  the native process harness. S alone removes that capture and keeps its
  attended private pipe; no captured link enters its manifest or evidence.
  S adds the trusted client seed, story, runtime LLM adapter and stopped checks.
- **Roles and capacity:** application charter 2 is coding/review's default;
  chat stories follow the actual chat template's executor, charter 1, whose
  120-second wait/ordinary run assertions remain. Each remote worker's
  `hub.turns` equals the configured agent's eight turns, preserving one
  slot. This corrects the first frozen S admission failure before sign-in;
  that run, its key and frozen experiment remain immutable. A corrected
  experiment requires a new capture and parent review.
- **Document-only checks:** configuration and mixed-document callers provide
  an explicit synthetic forge address. The shared cluster reader, case
  fixture and real-loop world remain case-2's landed code.
- **Host interface, joined:** `host.py::contract` accepts this manifest's
  absent stdin and public readiness only; it refuses link captures and
  placeholders. `Run.line`/`mark_ready` count the three declared engine
  listeners independently of the private stdout link. Startup requires both
  within the same ready120 deadline, in either stream order. The link's
  exact loopback origin must match clients; malformed, incomplete, repeated
  or missing links fail with cleanup. `start_child` delivers it once from
  memory through the sign-in pipe and discards it, with null stdin evidence.
  The story's full public origin is validated without prefixing or stripping
  paths. No link capture or placeholder is published. `collect_live` appends
  the actual D/E/R flags above and `stopped_check` requires engine/all trees
  empty. `collect_live` rechecks all six retained story records and records
  ROOT, CHILD and the full merge ID in its boundary event. Its source-based
  causal argument is `forge_route.rs`'s Finish → release → Settled chain:
  `continue_release` drains prior effects and deletes owned branches before
  Released, which maps to task Settled. The runtime root has no subsequent
  mutating script calls. This delegate settlement, final complete Open tree
  and the fake writer's flush-before-reply contract establish the pre-stop
  boundary. A checkpoint or story exit alone is insufficient; missing proof
  stays unknown. This is a source argument, pending an attended run.
- **Page copies, joined:** cluster-web inherits all five canonical service
  members and templates, including forge, coding, records-only setup, fake
  git and webhook readiness. It adds its bundle and browser. Look-live
  inherits that contract while preserving its live cue and chat-time
  overrides. Browser actions and collections retain their existing contract;
  the page owners' private-bootstrap replacement remains separate work.

`lab/tests/unit/test_smoke_assembly.py` checks document equality, explicit
seed/bootstrap adaptations, records-only setup and worker/agent admission
capacity. Page consistency tests compare the intended copies without
excluding the forge. Native configuration checks exercise the canonical
reader's synthetic capture path. These checks do not establish a successful
product story or prove the host's causal boundary on a real run.

Focused checks run through `heavy`: `test_ctl_output.py`, `test_smoke_assembly.py`,
`test_smoke_host.py` and `test_end_state.py` under `lab/tests/unit/`, plus the
existing native host canaries under `lab/tests/process/` when host changes
warrant them. They cover typed successful/failed owner results, fast-ended
discovery, incomplete evidence, shared deadlines, runtime isolation, private
bootstrap and live collection before stop/store copy/promises. Canonical
template equality and independent forge content/diff/promise checks remain.

This MVP is committed/pushed as a branch, with no gate, landing, Rust build,
experiment run, paid driver or campaign. The owner builds the exact-main
artifact after the listener lands and S freezes.
