# Audit: jig-client

2026-10-14, read only, by a sub-agent of the planner, at temper `cc2697cf` with jig `d91a89c4`, smith `dfc0fe57` and skein `b5d3a2c3` (temper's pins; jig's, smith's and skein's paths are relative to their pinned checkouts). Evidence the plan cites (development.md, 4.1 and 5.4): where it and landed code differ, the landed code wins. The design gaps it found are closed by the Markdown landing "Design: close the gaps the protocol plan's code audit found"; `README.md` here maps each finding to its increment.

I audited jig at d91a89c4 against the pinned checkout. Paths below are relative to `~/.cargo/git/checkouts/jig-aba39083bf77fefe/d91a89c`; skein paths are relative to its pin `b5d3a2c`. I read the pinned sources directly instead of the graph, which indexes newer mains. Most of what client.md assumes holds. The main gaps: the "client world kit" is a different crate than expected, the native client to copy depends on world-only code, and client.md's terminal and person assumptions have a few holes.

**1. Reference client roots**
- **ops's root** is `examples/ops/domain-client` (`jig-ops-domain-client`). Root modules: lib 24, address 15, owner 16, boundary 75, domain 196, route 318, parts 64, store 20, limits 82 (810 lines). The two connector sides are `infrastructure.rs` 970 (+544 +536) and `observability.rs` 684 (+317 +258); `tests.rs` is 1446. Each file opens with "Keep: … Replace: …" comments.
- **jig's test root** is `testing/jig-test-domain-client`: lib 26, address 11, owner 33, boundary 64, domain 157, route 221, parts 51, store 15, limits 50 (628 lines), plus `first.rs` 1011, `second.rs` 101, `family.rs` 238.
- **Shape (ops, `src/domain.rs`):**
  - `pub struct Domain { jig, infrastructure, observability, table: RequestTable<Owner, Ask> }` :12
  - `pub fn new(limits: &Limits, seed: u64) -> Domain` :19
  - `step(root, env: &Env<Limits>, event: Event, out: &mut Queue<Request>)` :43; `fire` :46; `next_deadline(&Domain) -> Option<Time>` :67
  - `take` :85: saves the table's records before sends, routes progress, then calls `reclaim` on the table and every child.
  - `shown(&Domain) -> u64` :113
  - `provider(root, jig_domain_client::signin::Provider) -> Result<(), Provider>` :183
  - `configure_navigation(root, Navigation) -> Result<(), Navigation>` :191
  - The test root exports no `configure_navigation` (`lib.rs:22`).
- **Owner** (`owner.rs:7`): `Jig(Token) | Observability(Token) | Infrastructure(Token) | Kept { scope: u64 }`. The test root has `Jig | First | Kept` (`owner.rs:29`).
- **Boundary** (`boundary.rs`):
  - `Ask{Jig(Box<JigAsk>), Infrastructure}` :12; `Answered` :18; `Action` :24
  - `Event{Start{address, kept: List<Record>, offset}, Went{place}, Act{action}, Answer{attempt, envelope, answer: Option<Answered>}, Jig(..), <side>(..), SignedOut}` :31
  - `JigOut{Read, Open, Continue, Close, Answered, Unknown}` :43
  - `Fact{Jig, TableFull, PartsFull, InvalidRecord, InvalidAnswer, UnknownConnector, KeptAnswered}` :53
  - `Request{Save, Erase, Send{attempt, key, ask}, Jig(JigOut), <side>(Out), Show{address, replace}, SignIn{provider, back}, SignOut, Fact}` :64
- **Store records** (`store.rs`): `VERSION = 1`; `Record{Jig(..), <side>(..), Table{version, record: RequestRecord<Ask>}}`; `Key{Jig(StoreKey), <side>, Table(RequestKey)}`.
- **Kept requests:** restored as `Owner::Kept{scope}` (`route.rs:64-80`). jig is then told `Event::Kept{scope, requests}` (:90). Retirement gives `KeptGone{scope}` (`domain.rs:146`, `route.rs:313`).
- **Parts by connector:** `parts.rs:6-48` matches `jig_ops_domain_connectors::connector(number)`, the shared typed enum that root.md §2 requires. The test root does the same with `Connector::of(number)` (`owner.rs:19`). Neither takes the number at construction. An unknown number gives `Fact::UnknownConnector`.
- **Leaving a page:** a side's address sends jig `Went{place: Place::Elsewhere}` and every other side `Event::Left` (`route.rs:121-149`).
- **Link and party** go to every side and to the table (`route.rs:185-197`).

**2. Reference view roots**
- **ops's view** is `examples/ops/view/src/root.rs` (217 lines; the crate is 4136 with 1566 of tests). The test view is `testing/jig-test-view/src/root.rs` (197).
- `View::new(limits, domain_limits, jig_words, obs_words, infra_words, project: u32)` :29. One phrasebook per connector: `words::Words::english(connector: u16, text: u32)` (`words.rs:512`). jig's words are `jig_view::words::Words{book, language, offset_minutes}` (`words.rs:21`), built from `phrases::english::english(text)` (`english.rs:1475`).
- `build(view, root, out: &mut Queue<Patch>)` :81:
  - returns early if `!render.stale(shown)`, then calls `begin()` on each bindings table;
  - opens `Role::Page`, then `jig_view::jig(b, root.jig(), NAV, words, jig_bindings) -> bool` :91;
  - on false it builds its own page, then calls `jig_view::after_page(...)` :103;
  - fills slots in two passes (`for pass in 0..2`, `builder.slots(pass)`): `Owner::Jig` slots get jig's cards, `Side{connector}` slots get the part or the navigation fragment, anything unfilled gets `unshowable` :186;
  - finishes with `render.finish`.
- The act entry point is named **`decode`**, not `act`: `decode(view, node: NodeId, act: Act) -> Option<Action>` :203. ops's `bindings::decode` treats Press, Toggle and Submit alike for an action meaning (`bindings.rs:60`).
- **jig-view API an application uses:**
  - `jig(b, &Domain, nav: SlotRef, &Words, &mut JigBindings) -> bool` and `after_page` (`pages.rs:125`, `:174`)
  - `Builder::slots(pass) -> Box<[Slot]>`, `splice(SlotId, Builder)`, `unshowable(SlotId, name)` (`builder.rs:260`, `:275`, `:372`)
  - `JigBindings::part(token)` (`binding.rs:117`) and `owner_part(token)` (`:140`)
  - `View` (render.rs): `new`, `stale`, `builder`, `finish`, `tree`, `act`, `address`, `external`
- **Navigation slot: confirmed.** `frame(b, client, nav: SlotRef, words, bindings)` (`frame.rs:13`) builds a Region named "Navigation". It holds jig's offered entries; `Destination::Application{..}` entries are drawn by nothing (:89). The slot itself is `b.slot(nav, "Loading navigation")` (:93).
  - Both references fix the navigation target at construction: ops by `project: u32`, the test view by `first::Address`.
  - Neither reads the frame's project. temper would read `root.jig().frame_summary().project` (jig-domain-client `domain.rs:189`, `frame.rs:85-116`).
- **The claim about forms and adoption: confirmed.**
  - jig-view leaves exactly three kinds of slot: navigation (`frame.rs:93`), `Part` (`cards/common.rs:93`) and `OwnerPart` (`proposed_task.rs:97`). None is for a form.
  - The project's Resources and Settings sections list resources (`pages/project/owner.rs:346`, :501, via `render` :323), each as an `OwnerPart` slot.
  - The confirmation maps `ActKind::Adopt` to `Refusal::NotOffered` (`confirmation.rs:2`, :305), which reads "This act is not offered" (`english.rs:965`).
  - jig's contract has no adopt request (`contract/asks.rs:3`).

**3. `jig-domain-client` API**
- `Domain::new(&Limits, seed)` `domain.rs:96`; `configure_navigation` :147 (refused after start or past `limits.navigation_entries`); `provider` :231.
- Free functions: `step` :568, `fire` :1138, `next_deadline` :1212, `reclaim` :1216, `max_out` :557.
- **Navigation** (`pages/navigation.rs`): `Navigation{home: Destination, entries: List<Destination>}`, with `Navigation::new(cap)` defaulting home to the inbox. `Destination{Jig(Address), Application{owner: u16, number: u32}}`. `offered()` shows Deployment only when `deployment_role.is_some()` (:72).
  - `home` is stored but never read: the shell's start address decides home.
- **Addresses** (`address.rs`): `Address{Inbox{project: Option<u32>}, Conversations{project}, Task{task, section}, Project{project, section: ProjectSection}, Deployment}`. `ProjectSection` includes Settings and Resources. `Place{Jig(Address), Elsewhere, Missing}`.
- **Events** (`boundary.rs:24`): Start, Hello{versions}, Kept{scope, requests}, KeptGone{scope}, Went{place}, Act, Ask, Progress, Answered, Read, Watch, Ended, ShowObjects, ObjectsLeft, SignOutLost, SignedOut, Frame{asked}.
- **Requests** (`boundary.rs:64`): Send{request, scope, ask}, Read, Open, Continue, Close, Show, Save, Erase, SignOut, SignIn{provider, back}, ShowPart{part, generation, scope: PartScope}, PartsLeft{generation}, Link{link}, Confirmed{party: Option<u64>}, Retention, FrameIs{asked, frame: FrameShown}, Answered, Unknown, Fact.
- **Link words:** `Link{Starting, Live, Offline}` (`link.rs:4`) renders as "Connecting", "Live", "Offline" (jig-view `english.rs:738-740`). Behind is "Behind: waiting for current facts".
- **Sign-in page:** shows one "Sign in with NAME" button per provider registered through `provider()`, and none otherwise (jig-view `pages/not_the_work.rs:15-26`).

**4. `jig-protocol-client-link`, `jig-shell-native`, the native client**
- **Link:** `Link::new(&Limits) -> Result<Link, Unusable>`, `step(now, Event, &mut Queue<Request>)`, `fire(now, out)`, `next_deadline` (`component.rs:34,71,101,50`). Limits come from `Limits::new(client, exchanges, deadline)` (`limits.rs:22`).
- **Link vocabulary** (`boundary.rs`): `Family{family: u16, version: u32, bytes}`, `Ask{Jig, Family}`, `Readout::Family{family, record: Option<..>}`, `Keeper{Jig, Table, Family(u16)}`, `Saved::Family{family, version, key, bytes}`. Requests include `Hello{family: Option<u16>, versions}` (emitted once per served owner, `component.rs:556-571`), `FamilyDelivery`, `Dropped{keeper}` and `ServiceCredential{party, credential}`.
- **`jig-shell-native`** (`src/shell.rs`, 367 lines; ordinary std Rust):
  - `Settings{engine: io::Addr, directory: Box<[u8]>, limits, io}` :13; `Shell::new` :48; `adopt(root: Fd, user, seed)` :81
  - `loaded() -> Option<Vec<StoreRead>>` :85, `events()` :88, `signing() -> Option<Signing>` :91
  - `sign_in(&mut self, printed: &[u8]) -> Result<(), Unlinked>` :96: parses the link (`sign_in.rs:21`; an IP:port and `jig/v1/sign-in/link?secret=`) and checks it names the engine.
  - `sign_out(&mut self)` :105, `down(link::Request)` :109, `close(&mut self)` :132, `iterate(now, wall)` :212, plus the Host-shaped accessors :308-349.
  - `Signing{SignedIn{exchange, party}, Again, Refused, Lost, SignedOut}`.
  - Files (`files.rs`): `OpenPrivate` on DIRECTORY (:163), then `token` (40 bytes) and `store` (:169-171). The token is written before `SignedIn` is reported. Exchanges are held until the files are ready. Close finishes a pending store write first.
- **"The native client of jig's client protocol world"** is `tests/client-protocol/src/native.rs` (360 lines; `NativeClient` :78, `new` :99, `impl Host` :250). It is not self-contained:
  - it needs `front.rs::Side` (lines 14-216 of 469), `front/persistence.rs` (110) and `family.rs` (103);
  - it drives the root through `jig_client_world::Test` and has the fake person built in.
  - Each iteration: `shell.iterate`; `loaded()` becomes `link::Event::Start{kept, versions: [Jig, Table, Family(1)]}`; signings; shell events into `side.event`; `outputs()`; start the root once signed in; root `fire`/`take`, `link.fire`; view build; the person acts.
- **Family routing (protocol/client.md §11):**
  - Root to link (`Side::wire` :140): family asks, reads and subjects are encoded into `link::Family` by family.rs. Every read and watch gets a fresh link token, kept in `BTreeMap routes/watched/subjects` (:24-28, :53).
  - Link to root (`Side::callback` :59): `Answered::Family` is decoded; `FamilyDelivery` and `Readout::Family` are decoded against the remembered subject.
  - The remapping exists because jig's tokens and a family's tokens share the link's one token space.

**5. Fake person, client world kit, joint world, costs**
- **`testing/jig-fake-person`** (lib 285, tree 154, random 154):
  - `Person::new(Box<[Step<A>]>)`, `next(&mut F: Face, now) -> Next<A>`
  - `Step{Go, Press, Type, Send, See, Gone, Reload, Pause}`; `Find::named / exactly_named / text / within`
  - `TreeFace::new(&Tree)`; `RandomPerson`
  - There is no Toggle step: Press always produces `Act::Press` (`lib.rs:202`).
  - It has no native front of its own. The "native person" is `NativeClient` plus the person, and the process tier's `Person` host reads the binary's stdout with raw `Op::PipeRead` (`testing/jig-test/tests/support/mod.rs:103-200`).
- **`jig-test-client-kit` is not the world kit.** It is 10 lines that re-export the test application's value-pricing helpers. The kit is **`jig-client-world`** (`tests/client`), ordinary std Rust with traits:
  - `trait App` (`harness/mod.rs:13`): types Root, View, Limits, Event, Request, Record, Key, Address; functions limits, new, start, went, step, fire, take, next_deadline, build, tree, act, duty→`Duty`, record_key, unsent, max_out, patch_limit, worst_case, seen, sent, received, card, and an optional window_changed.
  - `trait Front<A>` (:67): bind, wire, poll, next_due, observed, fault, reload, received, sign_in, sign_out, plus optional stored and restored.
  - Also `World::new(Settings, F)`, `run(Until)`, the referee and `Fault`.
  - jig's parts of an App delegate to `observe::jig_sent` :54, `jig_received` :199, `jig_card` :281, `jig_seen` :301, `jig_window_changed` :416.
  - The application reference is ops: `tests/ops/src/client/app.rs` (393 lines, `impl App for Ops` :143).
- **Joint world:** jig's own `tests/client/src/joint/front.rs::Joint` (565) over `session.rs` (1538) is hard-wired to jig's test application.
  - The application reference is ops's `tests/ops/src/joint/` (front.rs 1016 with `impl Front` :800, engine.rs 482, fixture.rs 274, infrastructure.rs 531, reads.rs 397), built on `jig_conformance::Harness`.
  - It reuses the public, application-independent pieces `jig_client_world::joint::{wire, packets, read_packets, transport, truth}`. `wire/*` is about 13k lines; only `wire/family.rs` touches the test application.
  - jig's 34 joint stories include "signing_in_and_starting_a_chat_on_the_engine" and "a_request_sent_twice_is_made_once_across_an_engine_restart".
- **Costs**, measured at d91a89c45. The files are in jig main 3eb97646 at `docs/development/plans/test-budgets/findings/`, not at the pin:
  - `jig-client-world`: 266 tests, 10.7 s summed in the gate, about 40 ms each, 4.6 s at `-j 1`.
  - Joint stories: about 15 ms setup plus 40–130 ms running each (6.3 s of the 10.7 s). "signing_in…" took 49.6 ms.
  - Client fixture 7.3 ms (phrasebooks 5.5 ms); a core world 3.5–6 ms; binary floor 2.2 ms.
  - Protocol world: 443 ms per test in the gate.
  - At the pin itself, `docs/development/plans/jig-protocol/tests-cost.md:168` gives `jig-client-world` 267 tests in 1.4 s in one process.

**6. jig-view `Tree`, roles, states, `Act`**
- `Tree::nodes() / get / find / bound / act / address / external` (`tree.rs:283-336`). `Node{id, role, size, name, states, styles, text: List<TextRun>, value: Option<Value{text, written}>, link, external, key, binding, slot, drawing}` (:190).
- **Roles** (:26), 35 variants: Page, Region, Heading{level: u8}, Paragraph, Text, List, Item, Table, Row, **Cell**, Term, Description, Button, Link, TextField, TextArea, Choice, ChoiceGroup, Check, Dialog, Status, Alert, Log, Tree, TreeItem, Tabs, Tab, Progress, Time, Code, Quote, Separator, Figure, Slot, Drawing.
  - Mismatch: `Cell` is missing from client.md 6.4's table. Everything else matches.
- **States** (:101): Disabled, Expanded, Busy, Current, Invalid, Pressed, Selected, Provisional, SubmitsOnEnter ("enter sends"). These match.
- **`Act`** (`binding.rs:16`): `Press | Input{value: Box<[u8]>} | Submit | Toggle`. `BoundAct{binding, act}` :29.

**Discrepancies**
1. "copied from the native client of jig's client protocol world … `route`: Step code: small total functions": the reference routing is world code with std `BTreeMap` token maps (`front.rs:24-28, 53`). temper's `route` needs bounded state or a token-tagging scheme.
2. Same native client: it ignores `Hello{family: Some}` (`front.rs:135`), so there is no reference for the forge's "out of date" page. Its `kept()` panics on family table records (`native.rs:179`; the restore that works is `persistence.rs:42`). It panics on `SignIn` (:229) and on sign-in Again/Lost/Refused (:280).
3. It starts the root only after `Signing::SignedIn` (`native.rs:276, 288`), so there is no reference for "by a token loaded". `Shell` also exposes no token-loaded signal.
4. "DIRECTORY refused: … what skein's io found unsafe or failed": `jig-shell-native` closes on a failed `OpenPrivate` and reports no reason.
5. "Signing out: DIRECTORY's token is erased": the shell writes an empty `token` file. `Shell::sign_out` drops the exchange token, so telling a lost sign-out (→ `Event::SignOutLost`) from a lost sign-in is left to temper.
6. "jig's client world: the client worlds' kit": the kit is `jig-client-world`'s harness (traits `App`/`Front`), not `jig-test-client-kit`. The joint-world reference to copy is ops's `tests/ops/src/joint` on `jig_conformance`, not jig's `Joint`. Most of the jig-contract translation is reusable from `jig_client_world::joint::wire`.
7. "taking jig's client limits from jig's small profile": jig has no small profile for client limits. Client limits exist only in world fixtures (`tests/client/src/test.rs:17`, `tests/client-protocol/src/fixtures.rs:94-105`, `tests/ops/src/client/app.rs:24`).
8. "Its connector number is given at construction": both references match on a shared typed enum or const (root.md §2), not a number given at construction.
9. "`act`" (5.1): the references name it `decode`.
10. 5.3 "`Adopt { project }` for the project the person is in, as jig's frame says it; with no project chosen, the slot holds nothing": no reference does this. Both fix the navigation target at construction.
11. "root.md, section 12: `ops`'s, and jig's test client root": §12 names only ops. The test root lacks `configure_navigation`.
12. 6.4 table: `Cell` is not listed. `* TEXT` for a status prints nothing for jig's loading and unshowable slot statuses, which carry only a name (`builder.rs:248, 372`). The renderer must fall back to the name.
13. Checks and choices: no jig view builds `Role::Check`, and the fake person sends `Act::Press`, never `Toggle`. The forge's bindings must accept Press, and Toggle from the terminal.
14. 4.1 resource words: on the project's pages, resources reach the slot as `OwnerPart` (`JigBindings::owner_part`), with jig already printing the connector number and literal segments. Grant patterns are printed by jig with no slot (`owner_authority.rs:118`), so the forge's pattern words have nowhere to appear.
15. 6.6 "pressing its provider" together with 8 "temper adds no sign-in provider": the sign-in page has a button only if temper's root calls `provider()` with the link provider's number.
16. 3.1 "home is the inbox": `Navigation.home` is never read; the shell's start address is the home.
17. "jig's process tests route a person's pipe reads" with `adopt_read_pipe`: jig's process test uses raw `Op::PipeRead` submissions. skein's `adopt_read_pipe` exists (`skein-io/src/layer.rs:158`).

Sizes of what client.md says to copy: the ops root's modules 810 lines; ops view `root.rs` 217 and `bindings.rs` 98; native.rs 360, the `Side` part of front.rs about 200, persistence.rs 110, family.rs 103. For the worlds: ops `client/app.rs` 393 and its `joint/` 2.9k.
