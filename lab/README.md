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
 "widths": [{"name": "desktop", "pictures": ["sha256:…"]},
            {"name": "phone", "pictures": ["sha256:…", "sha256:…"]}],
 "differences": [
   {"id": "composer-stack", "width": "desktop",
    "signatures": ["style/composer/font-family"],
    "tag": "style", "place": "mockup",
    "cites": "client/mockups/README.md, Where each value comes from",
    "seen": "temper's monospace stack adds Liberation Mono",
    "finding": {"class": "look", "title": "…"}},
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
  not taken. A list, since a state may draw a few pixels apart from one
  run to the next: the judge adds a digest once it has seen that
  nothing in the pictures moved. A scene whose pictures moved is judged
  again; one whose pictures stand is not (qa-web.md, 5.2).
- **A difference:** `id`, a name unique in the file; `width`; its
  `signatures`, the tails after `<scene>/<width>/` as the checks print
  them, each in at most one difference of its width; `tag`; `place`,
  `mockup`, `page` or `silent`; `cites`, the text it is judged by;
  `seen`, what the pictures show, in a line. Then by place: `mockup`, a
  `finding` (`class` and `title`) and the `expectations` written for it,
  if any; `page`, its `departures` and `expectations`, the scene's ids,
  at least one; `silent`, a `question`, at least two `options` and a
  `recommendation`.
- **Findings and questions are named** `<scene>/<id>`. The findings go
  to the lab owner's parent as they are made, which routes them through
  the director (qa-web.md, 9); the questions and the notes go to the
  user in one batch. `python3 -I lab/look_bar.py batch` prints the three
  lists from the files, each with its scene, width, signatures, the
  pair's paths and its citation.

### The bar

`lab/look_bar.py`, beside `look_pack.py`, reads a key's records in the
lab and the judgements, and prints the key's bar; `look-judge` prints it
after each key's verdict.

- **Each scene's verdict on each judged run:** `pass`, or `fail` and its
  signatures' count, and whether the runs' signatures are the same.
- **With `planted`** (the planted twin): each judged run's planted
  signatures, those present of all. Its other failures are the healthy
  key's or the fault's, and its pictures differ by design, so it has no
  unjudged list: the blind judge (below) judges it.
- **Without `planted`, the unjudged list.** A judged run's failing
  signature is judged when it is a look check's, `<scene>/<width>/<tail>`,
  its scene's judgements hold that width with the run's picture digest,
  and a difference of that width names `<tail>` as a finding or a
  question. Every other is listed once, with the runs it failed in and
  why: `departure-gone`, always; `not a look check` (the runner's own);
  `not judged` (no judgements for the scene or the width); `pictures
  moved`; `no difference`; `still failing` (judged the page's way, its
  departure or expectation not in the scene). Each check's `undecided`
  line is listed too. `unjudged: none` when nothing is.

### The planted chat's blind judge

The judge must name both planted differences (qa-web.md, 5.4) without
being told them. A fresh Claude session, whose working directory holds
only copies of a planted run's chat pictures (the pair and the regions'
pairs at both widths, named by width and region), is asked to compare
each mockup with temper's page and to name every difference it sees. It
sees neither `planted.css`, nor the experiment's name, nor the planted
signatures, nor the healthy run's chat, and reads the copies alone,
which its transcript shows. Its words and transcript are kept in
`judgements/look/blind/`; the bar holds when they name the changed
token and the composer's move.

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
gate; its shots are evidence, never goldens. `python3 -I
lab/look_bar.py bar --key KEY EXPERIMENT` prints a key's bar again
without a run.

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
counted; a judgements file refused for each rule above; and the batch.
The checks' own decisions are skein's, tested there on capture
fixtures.
