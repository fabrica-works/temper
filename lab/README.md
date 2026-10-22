# M1 lab pieces

**The experiment is not yet runnable.** These are S's independent pieces from
`docs/design/qa.md` (amended design `998a5f054`), reusable by H/P. No
`experiment.json` or configuration templates with guessed members are supplied.
Python 3.11+ standard library is the authorized temporary lab implementation;
none of this is product code or a temper world test.

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
heavy python3 -I -B -m unittest discover -s lab/tests/unit -v
heavy python3 -I -B -m unittest discover -s lab/tests/process -v
```

The unit tests use known binary content/trees, loss/incomplete/refused stories
and ctl output. Process tests use the real POSIX shell and `/bin/echo`,
`/bin/cat`, `/bin/true`, `/bin/false` canaries for guard/stream/status behavior,
and refusal of malformed story output. They introduce no fake product/harness.
No gate integration, cluster, model or paid calibration is run here.

Native validation through heavy: 18 unit tests passed in 0.003s and 14 process
tests passed in 0.258s. The process tests exercise real shells and canary
executables; they do not prove the product story, its containment or a full
cluster. No paid driver or calibration run has started.

Still required to finish S's canonical document/templates and runnable checks:

- First-run's fake CLI/documents, required TLS/root, cued tool-call scripts,
  CI content cue, widget ID and seed without NOTICE; fake git by argv[0].
- Cross-process deliveries, serve's bound webhook address available to
  reconcile, listener readiness and bootstrap link interface. No null webhook
  configuration, polling variant or shortened fallback is substituted.
- Fake-owned decimal u64 seed derivation and named 0600 key/token output
  interfaces. The runner will supply seeds and paths, never credential bytes.
- First-run's observation writer and complete flush reachable before stop,
  using skein `527480f3`'s encoding and explicit cumulative loss count 0.
  The promises judge landed at skein `4f13123c`; its command wrapper and
  Profile/Decision inputs remain first-run's, reviewed by github. Decision
  records must come from temper's own record, never the merge being judged.
  Decision 259 maps any retained breaches to exit 1, with fail lines and an
  undecided reason if present; exit 2 has reasons only, exit 0 no stdout.
- The designed private git `objects` route, including real request/response
  encoding and complete C/P trees. Then add the private decoder/route adapter
  and `checks/end-state`, stderr evidence and runner check result encoding.
- Director-routed `wait forges --until connected:1`, plus the
  runner's ready captures, collection and command/check contracts. Then render
  `forge.json`, `llm.json`, `config/temper.json`, agent/charter/worker files
  and `experiment.json`
  against those exact landed contracts, and run S's three attended seeds.

`add-key` is available at temper `dfc7cc70a` as
`add-key PROVIDER FILE --deployment --key KEY`. It reads a private regular
owned file and strips one final newline; it does not supply the fake's
pending key-output interface. Serve's directory form is available at temper
`65d2b3bea`: `temper serve --config CONFIG --data DATA --cache CACHE`, with
`temper.json` inside CONFIG. The lab supplies absolute paths to these three
directories. The fake forms and the forge wait remain dependencies in
qa.md, 11.1.
The wrapper/story expect the full printed sign-in URL without a second scheme
prefix, as qa.md, 5.1 specifies.
The fake Observation (`observe.rs`: Merged.commit, Moved.to, Refused) and Object
(`git.rs`: u64 counts, byte-path/content Tree) were source-read in skein's
`testing/skein-fake-forge-domain/src/`; the designed route is skein's
`fake-forge.md`, section 4. This is domain evidence, not a landed wire API.

Memory verification used `home-free-src-rust-temper`, generation
`2026-10-20T15:48:45Z`; relevant ctl paths matched metadata except the noted
changed docs/rendering/startup paths, which were source-read. Skein's graph
generation `2026-10-19T04:48:30Z` does not track those fake domain paths, so
their source is the evidence. No index was written or deleted.
The process cluster description and its templates live in [cluster/](cluster/).

# The look experiment

`look/` holds temper's pages to the mockups (`docs/design/draft/qa-web.md`,
sections 3 to 5): the gallery's states captured beside the mockups' at
1440×900 and 390×844, and checked. It has no driver and no process: a
seed action extracts the gallery printout, the collection is skein's
capture, and the checks are skein's look checks (skein's
`docs/design/lab-browser.md`, 4, 5 and 5.4). `look-planted/` is the same
with a fault (qa-web.md, 5.4).

## Its files

- `experiment.json`: `name` `look`, cost class `look`, one directory
  `page`, no processes, no driver, no faults;
  - the action `stage`: `/bin/sh {experiment}/stage {artifact.gallery}
    {dir.page}/gallery {experiment}`;
  - the collection `capture`: `{runner}/look/capture` with the settings'
    browser by its templates, `--mockups
    {source}/docs/design/client/mockups`, `--page {dir.page}/gallery`,
    `--out {collect.dir}/capture` and every scene;
  - the checks `look-structure` and `look-style`: `{runner}/look/check
    structure|style --capture {collected.capture}/capture` and every
    scene.
- `stage`: extracts the subject's tar into the page directory and requires
  exactly one `bundle/*/temper.css` (the bundle directory is named by its
  16-digit hex digest), failing loudly if zero or several match. When the
  experiment holds `planted.css`, it appends it to that stylesheet, the
  last word in the cascade for the pages' look.
- `scenes/<scene>.json`: `look-scene/1` files (below).
- **The planted twin,** `look-planted/`: `look/`'s files byte for byte,
  but `experiment.json`, which differs by `planted` alone, and
  `planted.css`. So with the stylesheet removed and `planted` cleared it
  is `look/`, tree and digest. The planted stylesheet changes one token
  and moves the chat's composer above its conversation. Its token is
  one whose regions agree with the mockup on the healthy page at both
  widths, so its signatures are new: `planted` lists
  `check/look-style/chat/<width>/style/<region>/<property>` and
  `check/look-structure/chat/<width>/order/composer` at each width.

## The subject

`lab subject --repository TEMPER --commit C --exclude lab --artifact
gallery=<gallery printout tar>`. The artifact `gallery` is the gallery's
printout at C: `look-judge` builds the release gallery, runs it into fresh
scratch space, and packs the output as one deterministic, uncompressed
USTAR tar. Names are sorted, relative and slash-separated; only regular
files and directories are allowed. Times and numeric owners are zero, owner
names empty, file modes 0644 and directory modes 0755. Names outside USTAR's
limits are refused. Thus every page's and stylesheet's bytes are in the
subject's digest: the same commit C gives the same key. The mockups come from
the subject's source (`docs/design/client/mockups/`), so a changed
mockup is a new subject and key (qa-web.md, 5.3); the scenes are the
experiment's, so a changed scene is a new experiment and key. The
gallery printout is the subject's artifact; the executable is used only
to produce it before the subject is made.

## The scenes

Each at `desktop` (1440×900) and `phone` (390×844, mobile), each side
ready by its `main`. The page is the gallery's state by its name
(`docs/design/client/browser.md`, 7.2), whose names
`tools/gallery/names.txt` keeps.

| Scene | Mockup | Page | Regions beyond banner, navigation and main |
|---|---|---|---|
| `chats` | `/chats.html?world=solo&bare` | `/chats-solo.html` | the chats' list, the composer |
| `chat` | `/chat.html?world=solo&task=T10&bare` | `/chat-solo-T10.html` | the chatbar, the conversation, the proposal, the composer |
| `inbox` | `/inbox.html?world=solo&bare` | `/inbox-solo.html` | the first entry, the "Needs you" and "Updates" lists |
| `task` | `/task.html?world=solo&task=T3&bare` | `/task-solo-T3.html` | the task's heading, the plan, the budget's card |
| `accounts` | `/accounts.html?world=solo&bare` | `/accounts-solo.html` | the first account's card |
| `first-run` | `/chats.html?world=first-run&step=1&bare` | `/chats-first-run-1.html` | the suggested requests, the composer |
| `adoption` | none | `/adoption.html` | the form, its first field, its submit button |

- **Regions** are found on temper's side by role and name, as a person
  finds them, and on the mockup's by role and name or, where its part
  has no role of its own, by selector. The view's names: the navigation
  `navigation "Main navigation"`; the chats' list by its heading; the
  chatbar `region "T10 · TITLE"`; the proposal `listitem "Proposal from
  T10"` (an `article` of that name in the mockup); the inbox's entries
  `listitem "KIND from TN"` and its lists by their headings. The
  mockup's side keeps a selector where its part has no role or name of
  its own (its chatbar, its inbox entries and sections), and for the
  navigation, which the phone mockups hide. The account card and the
  adoption form have no name yet: their regions stand on a named part
  inside them (the account's heading, the form's field and button).
- **Names** are the frame's and the controls' words (the navigation's
  links, Sign out, the composers' Send, a card's acts, the lists'
  headings), never content.
- **Expectations and departures** are the judge's (below). Expectations
  are few, since the base is the mockup as captured; `adoption`, with
  no mockup, is held to them alone. Each departure cites the section of
  the design text that says the page's way is meant.
- **Standing departures:** with the network closed, the mockups'
  Google Fonts stylesheet is refused and their families fall back to
  the lab's DejaVu, while temper draws its bundled faces; each such
  difference in `faces` or `font-family`, and the mockups' refused
  request in their trouble, is a departure citing qa-web.md, 11, until
  the mockups' fonts sit beside them. Nothing in `client/mockups/`
  changes.

## Judging

The judge (qa-web.md, 5.2) is the lab owner's step of a campaign, by
hand and outside the run: Claude reads each scene's three views at both
widths beside the design text of its page (`client/README.md`, the
page's section; `client/browser.md`, 2.5 and 7; `client/mockups/README.md`;
the gap tables its owners give). A sub-agent may draft a scene's
judgement; the judge checks it against the pictures and the text. Each
difference gets a tag (`layout`, `navigation`, `content`, `naming`,
`style`, `behaviour`) and a place:

- **the mockup's way:** temper's page is wrong, a finding (qa-web.md,
  9), classed `world` (words, order, a missing part or name) or `look`
  (appearance);
- **the page's way:** the mockup is behind. Departures at the
  difference's width, by signature or by outline entry, each citing the
  section that says the page's way is meant, and a note for the user.
  Without such a section the difference is a finding or a question,
  never a departure;
- **the text silent:** a question for the user, with its options and a
  recommendation. Its signatures go on failing, undecided.

### When the judge writes an expectation

The base is the mockup as captured, so a difference the mockup's way
already fails, and needs none. The judge writes one only where the base
is not the value meant:
- **a scene without a mockup** (`adoption`): its first judged capture's
  outline entries and the regions' values the judge holds right, so
  that it is held to itself (qa-web.md, 3.1);
- **a mockup's value not drawn as meant,** bound to its invented
  content or its refused fonts, where the text gives the value meant:
  the page is held to that rather than to the captured mockup;
- **a defect the pictures show that no signature names:** a region's
  value or an outline entry that fails on it, so that it signs.

### The judgements

`~/src/rust/tmp/lab/judgements/look/<scene>.json`, beside the lab's
keys: one file a scene, written by the judge, replaced when the scene is
judged again, and never read by the runner. `look-judgements/1`,
canonical JSON (skein's `lab.md`, 2.2), shown spread out:

```json
{"format": "look-judgements/1",
 "scene": "chat",
 "widths": [{"name": "desktop",
             "pictures": [{"digest": "sha256:…", "seen": "judged"}]},
            {"name": "phone",
             "pictures": [{"digest": "sha256:…", "seen": "judged"},
                          {"digest": "sha256:…",
                           "seen": "6 px, largest channel delta 3: corners"}]}],
 "differences": [
   {"id": "composer-stack", "width": "desktop",
    "signatures": ["style/composer/font-family"],
    "tag": "style", "place": "mockup",
    "cites": "client/mockups/README.md, Where each value comes from",
    "seen": "temper's monospace stack adds Liberation Mono",
    "finding": {"class": "look", "title": "…", "owner": "o/web-look"}},
   {"id": "fonts-phone", "width": "phone",
    "signatures": ["style/banner/faces"],
    "tag": "style", "place": "page", "cites": "qa-web.md, 11",
    "seen": "…", "departures": ["google-fonts-banner-faces-phone"]},
   {"id": "goal-deadline", "width": "desktop", "signatures": ["…"],
    "tag": "content", "place": "silent",
    "cites": "client/README.md, 11, A goal's page against its mockup",
    "seen": "…",
    "question": {"options": ["…", "…"], "recommendation": "…"}}]}
```

- **`pictures`:** the picture digests the width's judgement stands for.
  A width's picture digest is `sha256:` and the hex of the canonical
  JSON list of its shots' digests as the capture's record names them:
  the mockup's viewport (null without a mockup), the page's, then each
  region's mockup crop and page crop in the scene's order, null for one
  not taken. A list, since Chromium's raster is not exact from one run
  to the next: the antialiasing of a rounded corner may differ by a few
  levels, so one state may draw under several digests. The judge adds a
  digest once it has checked that nothing in the pictures moved, and its
  `seen` records the check: the pixels that differ and the largest
  channel delta. A scene whose pictures moved is judged again; one whose
  pictures stand is not (qa-web.md, 5.2).
- **A difference:** `id`, a name unique in the file; `width`; its
  `signatures`, the tails after `<scene>/<width>/` as the checks print
  them, each in at most one difference of its width; `tag`; `place`,
  `mockup`, `page` or `silent`; `cites`, the text it is judged by;
  `seen`, what the pictures show, in a line. Then by place: `mockup`, a
  `finding` (`class`, `title` and `owner`, the page's owner), classed
  `world` (words, order, a missing part or name), `look` (appearance) or
  `experiment` (the scene's or the gallery's, qa-web.md, 9);
  `page`, its `departures` and `expectations`, the scene's ids,
  at least one; `silent`, a `question`, at least two `options` and a
  `recommendation`.
- **Findings and questions are named** `<scene>/<id>`. The findings go
  to the lab owner's parent in batches as they are made, grouped by the
  page's owner, and it routes them through the director (qa-web.md, 9):
  a finding is filed once routed, and the lab's candidates of its
  signatures stay as they are. The questions and the notes go to the
  user in one batch at the end of the judging. `python3 -I lab/look_bar.py batch` prints the three
  lists from the files, each with its scene, width, signatures, the
  pair's paths (the page's own pictures for a scene without a mockup)
  and its citation.

A `mockup` or `silent` difference may also list the `departures` and
`expectations` written for the part of its line judged the page's way:
the line keeps failing by its other part, and is judged by its finding
or question.

A `mockup` difference may list no signature: a finding no look line signs,
seen in the pictures or the page's tree (words the look doesn't compare,
or a role or name a scene's finder holds), whose test belongs to the
page's own tiers (qa-web.md, 9). It judges no line.

### The bar

`lab/look_bar.py`, beside `look_pack.py`, reads a key's records in the
lab and the judgements, and prints the key's bar; `look-judge` prints it
after each key's verdict.
`look_bar.py bar` exits 0 when the bar holds, 1 when it does not, 2 on
input it cannot read.

- **Each scene's verdict on each judged run:** `pass`, or `fail` and its
  signatures' count, and whether the runs' signatures are the same.
- **With `planted`** (the planted twin), given the healthy key: each
  judged run's planted signatures, those present of all; and its
  signatures beside the healthy key's run in the same place of its three
  (the look draws nothing from a seed), as two sets, planted − healthy
  and healthy − planted. The judge confirms that each line of the first
  is the token's reach or the composer's move, and that the second is
  empty or explained. The first holds more than the planted six: the
  token also draws the banner's bottom border in every other scene and
  the task's cards' borders, and the moved composer moves the
  conversation too (at temper 1e1ca838, 6 + 26 + 1 = 33). Its pictures
  differ by design, so it has no unjudged list: the blind judge (below)
  judges its chat.
- **Without `planted`, the unjudged list.** A judged run's failing
  signature is judged when it is a look check's, `<scene>/<width>/<tail>`,
  its scene's judgements hold that width with the run's picture digest,
  and a difference of that width names `<tail>` as a finding or a
  question. Every other is listed once, with the runs it failed in and
  why: `departure-gone`, always; `not a look check` (the runner's own);
  `not judged` (no judgements for the scene or the width); `pictures
  moved`; `no difference`; `still failing` (judged the page's way, its
  departure or expectation not in the scene). The `undecided` lines are
  listed too: a look check prints one, and the run records it, for a
  part it cannot judge (a record's file missing or changed, a call it
  needs refused, a region's mockup part not found: skein's
  `lab-browser.md`, 5.4). A check that prints both exits 1, and the runner
  records its failures and its undecided alike (skein's
  `scripts/lab/runner/checks.py`, `command`): the bar lists both, and
  neither hides the other. `unjudged: none` when nothing is.

### The planted chat's blind judge

The judge must name both planted differences (qa-web.md, 5.4) without
being told them:
- **What it sees:** copies of a planted run's chat pictures, the pair
  and the regions' pairs at both widths, in a fresh directory outside
  `~/src/rust` with no `CLAUDE.md` on the path above it, under neutral
  names (the width and the region): no path it sees holds `planted`, a
  key, the experiment's name or a run's id. It sees neither
  `planted.css`, nor the planted signatures, nor the healthy run's
  chat.
- **How it runs:** `claude -p --restricted --tools Read`, a fresh
  session with auto memory off, from that directory. Its prompt lists
  the files and asks it to compare each mockup with temper's page and
  to name every difference it sees; it says nothing of a fault.
- **What is kept,** in `judgements/look/blind/`: the prompt verbatim,
  its words, and its transcript (`--output-format stream-json`).
- **Shown blind by a command:** `python3 -I lab/look_bar.py blind --dir
  DIR --transcript FILE --forbid WORD...` fails on a file under the
  directory whose path holds a forbidden word, a tool other than Read
  in the transcript, or a path the transcript names outside the
  directory, and prints the paths it read.
- **The bar holds** when its words name the changed token and the
  composer's move.

## Running it

```sh
lab/look-judge SKEIN EXPERIMENT...
```

From a clean temper checkout at its commit C: builds the gallery
through `heavy` (release), prints and packs its output in temporary space
removed after making the subject, and runs skein's `lab
judge` on each experiment named (`lab/look`, `lab/look-planted`):
three seeds a key (skein's `lab.md`, section 8), printing each key and
verdict, then the key's bar (above). More runs of a seed are `lab run
--seed N`. The lab is `~/src/rust/tmp/lab`, whose `settings.json` names
the browser and the `look` cost class. It runs on demand, never in the
gate; its shots are evidence, never goldens. Given both experiments,
the planted key's bar is beside the healthy key's. `python3 -I
lab/look_bar.py bar --key KEY [--healthy KEY] EXPERIMENT` prints a
key's bar again without a run.

## Its tests

`lab/tests/unit/test_look.py`, in the gate, fast and with no browser:
each scene canonical `look-scene/1` with the members above, its page a
name of `tools/gallery/names.txt`, its mockup's file in
`client/mockups/`; `experiment.json` naming every scene in its capture
and both checks; and `look-planted/` byte for byte `look/` but for
`planted.css` and `experiment.json`'s `planted`. `test_look_pack.py` checks
identical tar bytes and digests despite changed disk times and modes,
fixed archive metadata, and refusal of links, special files and names
outside USTAR's limits. `test_look_bar.py` checks the bar on records and
judgements written as fixtures: the picture digest; a signature judged
by a finding and by a question; each reason of the unjudged list; a
scene's verdicts the same on three runs and not; the planted signatures
counted, and the planted and healthy sets' two differences; a
judgements file refused for each rule above; the batch; and the blind
check on a transcript that stays in its directory and on ones that
leave it, use another tool or see a forbidden name.
The checks' own decisions are skein's, tested there on capture
fixtures.

# The cluster in the browser

`cluster-web/` is `cluster/`'s cluster with temper's browser bundle
served and a person's browser beside it, the hand
(`docs/design/draft/qa-web.md`, section 6; skein's
`docs/design/lab-browser.md`, section 8): the browser signed in as the
deployment's owner, and a scripted story, the smoke, driving it
through `browse`. It has no driver: it is the free smoke W2's later
experiments stand on, and it costs no model call.

## Its files

- **`cluster/`'s, copied:** `llm.json`, `chat.txt`, `agent.json`,
  `w1.*`, `w2.*` and `forge-1.*` byte for byte; `engine.json` but for
  `application.clients.bundle`, `"{proc.engine.dir}/bundle"` where
  `cluster/`'s is null. `experiment.json`'s processes `llm`, `engine`,
  `w1` and `w2` are `cluster/`'s member for member, but for `engine`'s
  `files`, which add the bundle's tree, `{"path": "bundle", "tree":
  "bundle"}` (skein's `lab.md`, 4.3). Copies, not names: an
  experiment's digest is its own directory's tree (skein's `lab.md`,
  2.3), so an experiment that named another's files would run with
  bytes its key does not hold, and the runner would need a new
  identity rule; a test holds the copies equal instead (below), and
  `cluster/` and its Rust reader stay as they are, with no Chromium.
- **`browser`,** after `engine`: `{browser.program} hand --chromium
  {browser.chromium} --fonts {browser.fonts} --scratch
  {proc.browser.dir} --out {dir.hand} --origin
  http://{proc.engine.ready.clients} --size 1440x900 --listen
  127.0.0.1:0`, `env` `{}`, `memory_bytes` null, ready by `listening `
  on its stdout, captured as `address`. At a desktop width: at 760 px
  and under, the banner's "Sign out" and the navigation sit in a closed
  disclosure, "Menu".
- **`dirs`:** sorted as `client`, `hand`, `spent`: `client`, the seed's
  `temper ctl` directory; `hand`, the browser's `--out` (its
  `trouble.jsonl` and its shots); `spent`, the directory of the `ctl`
  that presents the spent link.
- **`cost`:** `temper-m1-calibration`, `cluster/`'s (`qa.md`, 10.1): 60 s
  to be ready, 120 s a command, 10 s to stop.
- **`relink`, `smoke` and `checks/trouble`,** below.

## The seed

Each `{artifact.temper} ctl` action is `qa.md`'s, 5.1. In order:

| Action | Its argv | Its stdin | For |
|---|---|---|---|
| `sign-in` | `{artifact.temper} ctl {dir.client} sign-in -` | `http://{proc.engine.ready.link}` | the owner signs in by serve's printed link, which spends it |
| `add-key` | `{artifact.temper} ctl {dir.client} add-key 1 {proc.llm.dir}/llm.key --deployment --key cluster-web-llm` | | the fake model's key |
| `usable` | `{artifact.temper} ctl {dir.client} wait accounts --until usable --timeout 100` | | the account usable |
| `relink` | `/bin/sh {experiment}/relink {artifact.temper} {dir.spent}` | the spent link again | serve refuses it and prints a fresh one, the action's awaited line: `{"process": "engine", "stream": "stdout", "prefix": "http://", "capture": "link"}` (skein's `lab.md`, 4.5) |
| `browser-sign-in` | `{runner}/hand/sign-in {proc.browser.ready.address}` | `http://{action.relink.link}` | the browser opens the fresh link |
| `signed-in` | `{runner}/hand/browse {proc.browser.ready.address} wait heading Chats --exact --timeout 60` | | the chats page, signed in |
| `smoke` | `/bin/sh {experiment}/smoke {runner}/hand/browse {proc.browser.ready.address}` | | the story (below) |

- **Environments:** `browser-sign-in`, `signed-in` and `smoke` have
  `PATH=/usr/bin:/bin` for the hand launchers' `python3` shebangs; all
  other actions have empty environments.
- **A second `ctl` presents the spent link,** not the browser. The
  browser then signs in once, by the fresh link, so its trouble holds
  nothing the seed caused but its `sign-in` call's, which the entries'
  `during` names and the check leaves out (skein's `lab-browser.md`,
  8.7). A browser shown
  the spent link would show the signed-out page, and what that page
  logs after the call's load comes between calls, which the log cannot
  tell from the pages' own. It is also the product's way to a fresh link
  (`docs/design/client/README.md`, 11, "A fresh link from the client").
  The browser's page for a spent link is a story of its own, for the
  browser tier (`docs/design/client/browser.md`, 2.4).
- **`relink`,** POSIX sh: runs `TEMPER ctl DIRECTORY sign-in -` with
  its own stdin, and exits 0 when that exits 1 with `the sign-in link
  was refused` on its stderr; otherwise it exits 1, saying what it saw.
  The link is never an argument: it reaches `ctl` on stdin alone, and
  the browser through `hand/sign-in`'s.
- **The links in the record:** the first is `engine`'s ready capture,
  the second `relink`'s awaited capture, both in `run.json`, and both
  are spent before the smoke starts. What shows a run (a report, a
  finding's evidence) shows each query as `<link>` (skein's
  `lab-browser.md`, 8.6).

## The smoke

`smoke BROWSE ADDRESS`, POSIX sh: each step one `browse` call, its
output printed, and the story ends at the first that does not exit 0,
with its status, which fails the action (`action/smoke/exit:1`, skein's
`lab.md`, 7.1). Each expectation is a `wait`, so a missed one exits 1
(lab-browser.md, 8.8), 60 seconds at most but the first reply's, 90. The roles are the fake
person's; the names are o/web-look's words for the pages:

1. `type "text area" "Start a chat"`, its text `cluster-first-chat`,
   the fake model's cue (`cluster/llm.json`); `press button Start
   --exact`.
2. `wait log Conversation`, the chat's page; `wait text "First worker
   replied." --in log Conversation --timeout 90`: its reply, once its
   turn commits. A run is shown by its activity, and its words as each
   turn commits (`docs/design/protocol/llm.md`, 7; an agent process
   gives no provisional text, `client/tools.md`). The cue's turn writes
   the reply, then calls `wait`, which ends at the charter's
   `waiting_ms`, 60 s (`cluster/engine.json`): the reply shows 60 s
   after Start, most of a run's time, and the wait's 90 s leave it a
   margin. A shorter fixture or `waiting_ms` would only hide it.
3. `address`: the chat's, `/chats/<task>` on the origin, kept.
4. `go /chats`, `wait heading Chats --exact`; `go` the chat's path,
   `wait text "First worker replied." --in log Conversation`: its page
   by its address.
5. `reload`, the same wait; `shot`.
6. `back`, `wait heading Chats --exact`; `address`, which must be
   `/chats` on the origin, or the story exits 1.

## The browser's trouble

The check `trouble` (`qa-web.md`, 8.1): `/usr/bin/python3 -I -B -S
{experiment}/checks/trouble {collected.hand}/trouble.jsonl`, after the
live collection `hand`, `/bin/cp -R -- {dir.hand}/. {collect.dir}/.`.

- **Each entry fails `browser/<kind>/<page>`:** its kind as the hand
  logs it (an exception, a console error, a policy violation, a dialog,
  a request off the origin: skein's `lab-browser.md`, 8.7), and its
  page by its address's route, by temper's address codec
  (`docs/design/client/browser.md`, 2.3): `inbox`, `chats`, `accounts`,
  `chat`, `task`, `adopt`; `sign-in` for the link's route; `blank` for
  `about:blank`; `other` for any other path, and `off-origin` for an
  address off serve's origin. Never a task's or a project's number.
- **One line a signature,** its summary the entries' count and the first
  one's text, as the hand redacted it.
- **Left out:** the entries whose `during` is `sign-in`, the seed's
  calls.
- **Statuses** (skein's `lab.md`, 7.2): 0 with no entry left; 1 with
  one, an unreadable line's `undecided` printed beside it; 2, undecided,
  when the log is missing, or a line is unreadable and no entry is left.
- **The codec's table is copied** into the check, which names its
  source: `docs/design/client/browser.md`, 2.3, and the codec,
  `crates/temper-client-domain/src/address.rs` (`Address`, its `path`
  and its parse), each page by its variant's name in lower case. A page
  the codec gains names itself `other` until the table follows it.

## The subject and the command

```sh
lab/cluster-web-judge SKEIN
```

From a clean temper checkout at its commit C, as `look-judge` (above):
builds `temper` (release) and the browser bundle
(`scripts/bundle.sh --release`) through `heavy`, packs the bundle's
directory as `look_pack.py` packs the gallery, makes the subject
(`lab subject --repository TEMPER --commit C --exclude lab --artifact
temper=… --artifact bundle=…`), validates `cluster-web`, and runs
skein's `lab judge` on it: three seeds, printing the key and the
verdict. The lab's `settings.json` names the browser and the cost
class. It runs on demand, never in the gate; its transcripts are
evidence, never goldens.

## Its tests

- **`lab/tests/unit/test_cluster_web.py`,** in the gate, pure: the
  copies equal `cluster/`'s, `engine.json` but for its bundle's line,
  and the four processes but for `engine`'s tree; `browser` and the
  seed as above; the check's decisions: the page of each route and of
  the others, the seed's sign-in left out, one line a signature, the
  statuses, an unreadable line.
- **`lab/tests/process/test_cluster_web.py`,** on demand: `relink` and
  `smoke` under the real shell against stand-ins for `temper ctl` and
  `browse`: the refusal taken, every other exit refused, the smoke's
  calls in order, and a missed wait or a wrong address ending it with
  1.
