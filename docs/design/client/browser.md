# The browser

Provisional, 2026-10-05, revised 2026-10-19. temper's browser client: a
second shell of temper's client (`README.md`), beside the terminal, on
skein's browser parts. What its pages do is the client's (`README.md`;
`forge.md`), with the mockups' look (`mockups/`, reference material),
which this document takes as given; how a view runs in a browser is
skein's (skein's `ui-web.md`), and how a client of jig's does is jig's
(jig's `protocol/browser.md`). It is built in two milestones (section
6). What is still open is section 5.

## 1. In one page

- **A second shell of the same client.** temper's client root, its view
  and the forge's family (`README.md`, sections 3 to 5), compiled to
  WebAssembly under a browser shell of temper's,
  `temper-client-shell-web`, on skein's browser parts (`skein-ui-web`;
  skein's `ui-web.md`) and jig's client adapter and sign-in part
  (`jig-client-shell-web`; jig's `protocol/browser.md`, 5.1), with
  neither jig's look nor its reference UI. The adapter carries jig's
  default HTTP mapping over fetch (`jig-client-protocol-http`). temper
  writes no web wire and no web protocol of its own: the client's
  records cross as jig's wire carries them, binary records from a schema
  per owner, never JSON (jig's `protocol/client.md`, section 3).
- **The pages temper has,** in both shells: the inbox, the chats, a
  chat, a task's page and the adoption form (`README.md`, 3.5). A page a
  later plan adds to the view (the board, the changes, the accounts) is
  in the browser once it is in the view, with its look and, where it
  has one, its browser-only story (4.2).
- **No framework.** The client and its view are step code under the
  subset in a browser too (skein's `ui.md`, section 1).
- **The view tree is the fake DOM.** Below the browser, every world
  reads skein-ui's tree by role, name and text through skein's fake
  person (skein's `ui.md`, 17.1): no browser, no WebAssembly and no fake
  DOM library.
- **Served by temper's server, from one origin.** `temper-service` serves
  the page and the bundle on its clients' listener (skein's `ui-web.md`,
  4.4), beside jig's clients' component under `/jig` (`README.md`,
  section 8), from the directory its configuration names, with jig's
  example guards and the page's policy, which is temper's: a strict
  content security policy, no frames, no CORS (jig's
  `protocol/browser.md`, 12.1 and 12.2). A configuration that names no
  bundle serves no browser, and the terminal works as before (2.2).
- **Signed in by the printed link,** opened in a browser on the engine's
  machine: jig's sign-in part spends it, the credential is the
  mapping's cookie, and a reload signs in again by it (2.4).
- **Browser tests only for what a browser alone shows.** temper's
  stories are the worlds' and the real loop's, through the terminal
  shell (`../domain/testing.md`, 2.1, 2.4 and 6.3). The browser tier proves
  the bundle under its policy, signing in by the link and the cookie,
  addresses, the composer's keys and jig's attack stories that need a
  browser, and nothing a world can show (section 4).
- **A real browser at the top, driven from Rust,** by `skein-browser`
  (section 4).

## 2. What temper brings to the browser

- **Its crates as they are.** The client root, the forge's side, the
  view and `temper-client-protocol-wire` are step crates under the
  subset, which build for the host and for wasm32 alike; the terminal
  and the agents' tools are not part of the bundle.
- **Its look:** after the mockups (`mockups/style.css`), as temper's
  style tokens and a stylesheet over skein's (skein's `ui-web.md`, 3.2
  and section 12): 2.5.
- **Its pages,** the client's (`README.md`, 3.5), each at an address of
  its own (2.3); and the forge's parts, from `forge.md`, as the forge's
  side reads them.
- **Signing in:** by the link `temper serve` prints (2.4). The forge as
  people's sign-in provider is temper's own routes (`README.md`,
  section 11).
- **The toolchain:** the standard library for `wasm32-unknown-unknown`,
  skein's bundler and the glue it names (skein's `ui-web.md`, section 4),
  and Chromium for the browser tier.

### 2.1 The shell

- **`temper-client-shell-web`** is a `cdylib` for
  `wasm32-unknown-unknown`, ordinary Rust as a shell is (skein's
  `ui.md`, section 2), copied from jig's reference browser shell,
  `jig-client-test-shell-web` (jig's `protocol/browser.md`, 3.1), with
  temper's client root, view and records in place of jig's test
  application's, and none of jig's look (`jig-client-shell-web`'s
  `look`, `jig.css`). Its modules:
  - `lib`: the start (skein's restart part; jig's sign-in part taking
    the link from the address before anything is sent; the small store;
    the page with temper's look; the DOM) and the turn of skein's
    `ui-web.md`, 5.2; and, as jig's reference shell has it, the test
    address `/test/panic`, at which the shell panics at start, for
    skein's `panic` story (4.1);
  - `host`: the client host: temper's client root and view, jig's
    mapping under `/jig` and its stream; the root's requests to jig's
    adapter (`jig-client-shell-web`'s `adapter`) and skein's transport,
    and their reports back to the root;
  - `records`: the small store's records, `jig-client`'s alone; from the
    adoption form (section 6), a pending adoption is among them, a kept
    request whose request `temper-client-protocol-wire` encodes
    (`README.md`, 4.6);
  - `limits`: the client's limits in a browser, derived as the
    terminal's are (`README.md`, section 2), and their sum.
- **It decides nothing.** What a page shows and does is the domain's
  and the view's, step code under the subset; the shell orders the
  platform's work (jig's `protocol/browser.md`, 3.1).
- **The failure page:** past the restart part's limit, the page says
  "The application failed.", in temper's words, since it takes none of
  jig's look; it starts nothing and keeps the small store (jig's
  `protocol/browser.md`, 5.1).
- **The small store** is the browser's session storage, under the
  prefix `temper.`, holding `jig-client`'s records alone, as the
  terminal's file does (`README.md`, section 7); from the adoption form
  (section 6), a pending adoption is among them, a kept request
  (`README.md`, 4.6). Each load of the page is a client of its own, with
  its own stream (jig's `protocol/browser.md`, section 10).
- **What the terminal says in lines of its own** (`README.md`, 3.4 and
  6.6): another party's kept work and kept work dropped have no place
  in the browser, whose small store lives as long as its tab, of one
  person (section 5). A service's credential is never shown: temper's
  client makes no service (`README.md`, section 11).

### 2.2 Serving

- **The routes of the clients' listener,** temper's, in this order,
  over jig's mapping (jig's `protocol/browser.md`, section 12) and
  skein's serving of a bundle (skein's `ui-web.md`, 4.4):
  1. what jig's default mapping claims, under `/jig`, to jig's clients'
     component: every request there but a `GET` of the printed link's
     route, which the mapping leaves to the page, answered with the
     page's policy and `Referrer-Policy: no-referrer` (jig's
     `protocol/browser.md`, 12.1 and 12.2);
  2. with no bundle configured, every other path not found, as
     `README.md`, section 8 says;
  3. a request other than `GET`: `405`;
  4. `/bundle/<digest>/…`: the bundle's files, immutable; any other path
     under `/bundle/`: not found;
  5. `/robots.txt`, the bundle's one root file (skein's `ui-web.md`,
     3.3, step 5; jig's `protocol/browser.md`, section 12), disallowing
     every path;
  6. any other `GET`, the printed link's route among them: the page,
     with its ETag and `no-cache`, and `304` on a matching
     `If-None-Match` (skein's `ui-web.md`, 4.4).
- **The page's policy,** on every answer temper gives: jig's reference
  policy as it stands (jig's `protocol/browser.md`, 12.1): `default-src 'none';
  script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src
  'self'; font-src 'self'; connect-src 'self'; base-uri 'none';
  form-action 'none'; frame-ancestors 'none'`, with `nosniff`,
  `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin` and `Referrer-Policy:
  no-referrer`. skein-http's own refusals, `400`, `413` and `431`, given
  before temper sees the request, carry none: they hold no page to frame
  or sniff. temper adds no origin: no font, picture or script from
  elsewhere. The mockups' optional web font is not used; the look takes
  the system's monospaced font.
- **jig's example guards, as they are** (`README.md`, section 8): the
  Host allow-list holding the listener's own address; a cookie without
  jig's browser header refused.
- **The bundle** is skein's bundler's output (skein's `ui-web.md`,
  section 4): the shell's module, skein's stylesheet and temper's
  (2.5). `scripts/bundle.sh` builds it, from the module built for
  wasm32, into a directory it prints. `application.clients.bundle`
  names that directory, or is `null` for none
  (`../protocol/configuration.md`, section 3). `temper serve` reads it
  at start, whole, into memory, within the limits `clients.bundle_files`
  and `clients.bundle_bytes` (`../protocol/configuration.md`, section
  7), and refuses the start past them, or when the directory holds no
  bundle (its page and its digest's directory). A new bundle is served
  after a restart; a tab on the old one is answered not found for its
  files, and a reload loads the new (skein's `ui-web.md`, 4.4).

### 2.3 Addresses

- **Each page's address is a path,** by temper's address codec, a
  module of `temper-client-domain`, step code, total and bounded:

  | Page | Path |
  |---|---|
  | `Inbox` | `/` |
  | `Chats` | `/chats` |
  | `Chat { task }` | `/chats/<task>` |
  | `Task { task }` | `/tasks/<task>` |
  | `Adopt { project }` | `/projects/<project>/adopt` |

  `<task>` and `<project>` are the task's and the project's numbers in
  decimal. A later page's path comes with it, in this shape
  (`/projects/<project>/board`, `/projects/<project>/changes`,
  `/accounts`). A page's filters (the inbox's project, kind and goal)
  are not in its address (section 5).
- **A path that is no page** opens the inbox, and the address is
  replaced with `/`. The codec never prints a path under `/jig`,
  `/bundle/` or `/test/`, nor `/robots.txt`.
- **Moving** is skein's (skein's `ui-web.md`, section 9): a link pressed
  is a move within the client, with a new history entry; back and
  forward are moves too, each entry keeping its scroll; a reload, or an
  address typed or opened, is a new client at that address (jig's
  `protocol/browser.md`, section 10). That a client opens at the inbox
  (`README.md`, 3.4) is the terminal's: in a browser the address is the
  browser's, and the client keeps no record of its own.

### 2.4 Signing in

- **The printed link,**
  `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`, opened in a
  browser on the engine's machine (`README.md`, section 7): temper's
  server answers its `GET` with the page (2.2); jig's sign-in part takes
  the secret and replaces the address with `/` before anything is sent;
  the client signs in with it (`SignIn { with: Link(secret) }`), and the
  mapping's answer sets its cookie, which the client never sees (jig's
  `protocol/browser.md`, section 11). The inbox follows.
- **A reload, or another tab,** signs in with `Transport`, by the cookie
  (jig's `client/domain/client.md`, section 12).
- **Signed out,** by the banner's "Sign out" (jig's sign-out clears the
  cookie) or by a link spent or expired: the main is the view's
  signed-out paragraph (`README.md`, 5.1), in the browser's words:
  "Signed out. To sign in, open a sign-in link `temper serve` printed."
  The view takes its shell's words at construction; the terminal's stay
  "Signed out." with its own line (`README.md`, 6.6).
- **One person, on loopback,** as `README.md`, section 7 says. A browser
  on another machine is open (`README.md`, section 11).

### 2.5 The look

- **temper's style tokens,** under the owner's prefix `temper-`, on the
  nodes the mockups style: the frame's banner, navigation and main; the
  cards; the conversation's turns; the composers. The view sets them
  (`README.md`, 5.1); the terminal prints none.
- **The bundle's stylesheets:** skein's `skein-ui.css`, then
  `temper.css`, which takes its rules from `mockups/style.css`; the
  mockups are not shipped. `temper.css` holds the mockups' palette as
  variables under the prefix, the system's monospaced font, the compact
  top navigation and quiet borders. Nothing a person must know is said
  by style alone (skein's `ui.md`, section 8). The mockups' icons are
  left out: each control is named by its words.
- **Light only:** the mockups' palette is light, and `temper.css` sets
  `color-scheme: light`, where `skein-ui.css` follows the system's dark
  preference. A dark palette is open (section 5).
- **The composers** (the chats' and a chat's) are text areas with
  skein-ui's `SubmitsOnEnter`: Enter sends, Shift+Enter is a new line
  (skein's `ui-web.md`, section 7), which the view allows in a
  composer's words, counted within its bound (`README.md`, 5.1). They
  grow with their words and shrink after sending by `temper.css` alone
  (`field-sizing: content`, from one line to a bound), with no
  component and no script of temper's; a browser without it shows an
  area of fixed height that scrolls.
- **A run's live text** is a patch like any other: the chat's
  provisional turn ("Writing…", `README.md`, 3.5) changes in place;
  skein's page keeps focus, caret and scroll across it, and the
  conversation, a log, keeps its end in view while the person has not
  scrolled away (skein's `ui-web.md`, section 7).

## 4. The browser tier

- **The driver is skein's.** `skein-browser` (skein's `browser.md`) runs
  headless Chromium as a child and drives it over the DevTools protocol
  on a pipe, from the loop beside the engine.
- **The person's browser face is skein's** (`skein-fake-person-browser`;
  skein's `ui-web.md`, 14.2): the same queries and acts as on the tree,
  made over Chromium's accessibility tree, and the browser's own: the
  current address, read from the navigation history, and keys pressed
  with Shift.
- **What it runs against:** `temper serve`, a child of the harness's
  loop, on a scratch directory, from a configuration the test writes
  with the bundle named: no Forgejo, no runner. The first milestone's
  stories need no answer of the forge's or the LLM's, so its forge and
  its LLM endpoint are unreached, on closed loopback ports, as jig's
  tier serves (jig's `protocol/browser.md`, section 15). The fake
  forge, hosted on loopback in that loop by skein's forge peer
  (`../domain/testing.md`, 4.2), and skein's fake LLM peer, as the
  process tier has them (`../domain/testing.md`, 2.6), join with the
  second milestone's first story that needs them (4.2).
- **Only what a browser alone shows.** temper's stories (what a page
  shows, what an act sends, a chat answered, a proposal accepted, a
  repository adopted, a run's live text) are the joint world's
  (`tests/client`; `../domain/testing.md`, 6.3) and the real loop's, through
  the terminal shell (`../domain/testing.md`, 2.4); they never run again in a
  browser. A story is in this tier only when no world can show it: the
  bundle, the browser's platform (the address, history, the cookie,
  keys) or an attack that needs a browser. Each names what only the
  browser shows. What skein's browser parts prove for every application
  (the mapping of roles, a patch keeping focus, caret and scroll, the
  store, the bundler) and what jig's prove for every client of jig's
  (jig's `protocol/browser.md`, section 15) are not proved again.
- **Every story checks the policy too:** skein's harness fails a test
  on a policy violation, an exception or a native dialog (skein's
  `ui-web.md`, 14.2).

### 4.1 The first milestone's stories

1. **Signing in by the printed link:** the inbox shows, and the
   address reads `/` once it does; a reload on a chat's address shows
   that chat, signed in by the cookie; a link spent shows the browser's
   signed-out paragraph (2.4). That the secret leaves the address
   before the first request is held by the shell's start order (2.1),
   and is not tested.
2. **Addresses:** skein's `back_and_forward` (skein's `ui-web.md`,
   14.2), from the chats to a chat by its link and back; an address
   that is no page opens the inbox, and the address reads `/` once it
   shows (2.3).
3. **The composer:** in the chats, Enter starts a chat with the words
   once, and Shift+Enter makes a new line in them (2.5).
4. **A panic:** skein's `panic` (skein's `ui-web.md`, 14.2), which
   checks the shell's wiring of the restart part, at the test address
   `/test/panic` at which the shell panics at start, as jig's reference
   shell has it (2.1).
5. **jig's attack stories that need a browser,** from
   `jig-client-fake-person-browser`, against `temper serve`: a
   cross-site request from another port and from another host does
   nothing; a framed page shows nothing (jig's `protocol/browser.md`,
   section 15). The rebound Host, which needs no browser, stays in the
   process tier (`README.md`, section 10).

Not in the tier: skein's `typing_while_patched`, which takes a second
person, and `write_refused`, which takes a test build's bounded small
store; what they check is skein's page and store, which skein's tier
proves.

### 4.2 Later pages

A page gets a story here only for what of its own a browser alone
shows. The inbox, a task's page and the adoption form have none beyond
their addresses: one story opens each present page's address directly
and reads its heading. A later page that has such behaviour brings its
story with it: the accounts page's sign-in address, an external link
that opens a tab of its own.

### 4.3 Its budget and the gate

- **A suite of its own,** outside the focused and fuzzy suites
  (`../domain/testing.md`, section 2), in its own crate
  (`tests/client-browser`, `temper-client-browser-world`), run by
  temper's gate with a nextest profile `browser`, one test at a time,
  on the bundle the gate builds once, with the dev profile.
- **A guide: at most 12 tests, and about 60 seconds** on the slowest of
  the machines that run temper's gate. Past it, a story moves to a world
  or goes, and the budget stays.
- **Chromium** is the machine's, at the path its owner configures
  (skein's `browser.md`), which skein's and jig's gates read from
  `SKEIN_TEST_CHROMIUM`, and temper's gate too. A missing Chromium or
  bundle fails the suite, and never skips it (skein's `ui-web.md`,
  14.1).
- **The client's crates for wasm32:** the gate runs clippy for
  `wasm32-unknown-unknown` over `temper-client-domain`,
  `temper-client-view`, `temper-client-protocol-wire` and
  `temper-client-shell-web`, so a change that stops them building for
  the browser fails the gate.

## 5. Open questions

- **Kept work in the browser** (`README.md`, 3.4 and 6.6): another
  party's kept work, and kept work dropped at start, which the terminal
  says in lines of its own. Default: not shown; the small store lives
  as long as its tab, of one person.
- **A page's filters in its address:** the inbox's project, kind and
  goal. Default: not in the address; a reload shows the page
  unfiltered.
- **The bundle in the binary,** so that `temper serve` needs no
  directory. Default: a directory the configuration names (2.2).
- **A dark palette,** following the system's preference as
  `skein-ui.css` does. Default: light only (2.5).
- **Other browsers** than Chromium are `skein-browser`'s question
  (skein's `browser.md`).

## 6. In two milestones

- **The first: one path end to end, sign-in, the chats, a chat.**
  Everything every page shares: the shell (2.1), serving and the policy
  (2.2), the address codec with every present page's path (2.3),
  signing in (2.4), the frame's look and the chats' and a chat's (2.5),
  the bundle and the gate's checks (4.3). A chat's own, drawn by
  `temper.css`: the composer's keys and growth, a run's live text, a
  proposal accepted or rejected in place. The tier's stories of 4.1.
  Upstream, in skein, before them: a click inside a button or a menu
  item finding it (skein's `ui-web.md`, section 6), since temper's pages
  click buttons; `back_and_forward` awaiting a page that holds its
  server's state before its first move (skein's `ui-web.md`, 14.2),
  since the first page signs in by the printed link; and the fake
  person reading the current address and pressing a key with Shift
  (skein's `ui-web.md`, 14.2). Done when temper's gate passes 4.1's
  stories.
- **The second: the other pages.** The inbox, a task's page (a goal's
  and an agent's task's) and the adoption form, each with its look
  (2.5), and 4.2's story; the forge peer and the fake LLM peer join the
  tier with its first story that needs them (4; the forge peer
  forge-kit's). Done when temper's gate passes it.
- **What it touches in temper:** `temper-service`'s clients' server and
  its configuration and limits (2.2); `temper-client-domain`'s address
  codec (2.3); the view's style tokens, its composers' `SubmitsOnEnter`
  and its signed-out words (2.4, 2.5); the new shell and the tier.
