# S assembly and integration review

This is the no-driver NOTICE smoke from `docs/design/qa.md`, sections 4–8.
It is an assembly draft, awaiting case-2's canonical forge/reader contracts
and the attended host join. No successful product run is claimed. Use the
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
wait/run windows remain. NOTICE adds chat delegation to procedure 2, coding
charter 2, exact write/shell/change results and a review at the named head.
The landing fixture supplies grants, CI/up-to-date/exact-review rules and
capacities, without its shortened forge polls or rate window. Typed limits
and template checks are not proof of runtime admission or measured minima.

The five trusted seed actions are sign-in, add-key, account wait, forge wait,
adopt. Adoption precedes the story's first chat. `tools/ctl` and `smoke/story`
take `TEMPER CLIENT_DIRECTORY PUBLIC_ORIGIN`; the last argument is exactly
`http://HOST:PORT/`, never a sign-in link or path. The seed's sign-in command
declares no stored stdin: the host supplies the newline-terminated credential
through its private pipe. Seed waits200/story220 stay within command240;
ready120/stop20 and existing service graces and memory declarations stay.

The required evidence order is live observations with an independently
confirmed causal checkpoint boundary, then live C/P object reads and raw
request/response evidence; confirmed engine exit and empty trees; copied
`data/state/store`; then the existing `fake forge check` using that copy.
The manifest lists the commands, while the attended host owns this order.
The generic runner's collect/check-before-stop path cannot execute this S.
No checker opens a live store. An ended chat or EOF alone never confirms
the boundary; absent proof must pass `unknown` to the helper.

The host appends the helper's existing `--boundary confirmed|unknown`,
`--deadline-monotonic D`, `--evidence-dir E`, and `--result R` arguments to
`checks/end-state OBSERVATIONS FORGE_ADDRESS`. D is the actual absolute host
command deadline; E is a fresh existing evidence directory and R its result
path. The helper retains its own 120-second total cap. Preserve the raw
observation file, selected complete explicit zero-loss checkpoint prefix,
causal proof, C/P/tree/NOTICE stderr, object request/response bytes, result
and exit status. The promises command derives independent decisions from
temper records; there is no alternate credential or decision input schema.

Before a run, join and review these contracts:

- **Case-2 cluster and reader:** QA changed canonical `agent.json`,
  `llm.json`, `chat.txt`, added `coding.txt`, changed `engine` limits/LLM/
  charters/rules/project policy and `application` landing/charter/limits,
  and only the worker limits in `w1.json`/`w2.json`. Canonical
  `experiment.json` only adds the coding file and removes link readiness.
  Its existing processes, forge fixture files and engine
  `application.forges`, `identities`, `setup`, `webhooks`, plus worker
  `application.workers`, remain at the base. QA's forge-address references
  cannot render there until case-2 supplies its forge predecessor/capture.
- **S's anticipated case-2 portion:** `forge.json`, the forge member/seed/
  observations contract, LLM's forge predecessor and seed, engine webhook
  readiness, the four setup fields above and worker `{artifact.git}` are
  inherited draft contracts only. The forge seeds widget first and CI on
  NOTICE; engine setup omits old file credential members and uses the fake's
  admin-token handover. Records-only local start already exists at
  `c921947b5`; bound webhook registration is a producer join. Compare with
  case-2's landed canonical files before running. Do not reconcile externally
  or reinstate old credential fixtures to work around this dependency.
- **Shared process reader, case-2 owned:**
  `crates/temper/tests/support/cluster.rs` needs the subject materialized as
  `git`, the corresponding artifact binding, fake seed bindings, forge
  readiness before rendering QA engine/LLM templates, and document-only
  captures sufficient for those references. S's client directory/actions
  remain the attended host's; canonical cluster still has none. Existing
  read/document callers must supply the added captures. In particular,
  `crates/temper/tests/configuration.rs::cluster_documents` and
  `process.rs::a_mixed_document_exits_1_naming_its_path` currently supply only
  LLM/worker captures; configuration's memory match also only knows the old
  four members. No reader or caller was edited or tested by QA.
- **Host interface, host owned:** the inspected `temper-host` draft's
  `host.py::contract` still requires a `proc.engine.ready.link` capture and
  sign-in stdin template, and expects a bare clients endpoint for story.
  This assembly supplies no link capture/template and supplies the complete
  public origin. `Run.line`/`mark_ready` must count only declared public
  readiness while privately retaining the link. `start_child` already
  injects sign-in from memory; its story argument normalization must accept
  the complete public origin. `collect_live` already appends the actual
  D/E/R flags above and `stopped_check` requires engine/all trees empty.
  Its causal completion claim still needs review against the producer's
  settled-effects/observation-flush contract: story exit plus a checkpoint
  without that implication is insufficient. Keep unknown if unproven.
- **Page owners:** canonical byte-copy assertions in
  `lab/tests/unit/test_cluster_web.py::FilesTests` will need routing to
  `lab/cluster-web/{agent.json,llm.json,chat.txt,engine.json,w1.json,w2.json,experiment.json}`
  and its coding-file addition. The cluster-web → look-live copy chain in
  `lab/tests/unit/test_look_live.py::FilesTests` must then follow, including
  its inherited process contracts and added coding template. Those files
  and tests are not changed or suppressed here.

Targeted verification on 2026-10-23, through `heavy`: 45 tests passed in
0.357 s (10 assembly, 16 end-state, 5 ctl-output, 14 shell/canary). A separate
six-test downstream file check took 0.007 s: the four look-live checks passed;
`test_cluster_web.FilesTests.test_copies_and_engine_line` failed for
`llm.json`, `chat.txt`, `agent.json`, `w1.json`, `w2.json` and the engine
comparison, and `test_experiment` failed for the engine process contract
(coding file and removed link readiness), seven failures retained. Actual
skein lab validation with symbolic host facts accepted S's manifest, and its
real template renderer rendered every process JSON. This does not prove host
facts, startup, configuration admission or the product story.

Neither owner lands before reciprocal cluster/reader diff review. After
those joins, the parent rechecks and lands through `land`, then runs the
three attended S seeds. This task performs targeted Python checks only;
there is no build, gate, real run, landing, paid driver or campaign.
