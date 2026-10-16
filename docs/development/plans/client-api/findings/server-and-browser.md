# The server and the browser at the base (steps 5 and 6)

The code audit of jig at `a3b53546` and skein at `ecccbac`, 2026-10-15,
by a read-only sub-agent, from source (the graph was stale). jig pins
skein `f12f29d`, three Markdown-only commits behind skein's main, so jig
already has skein's browser code. Nothing was built.

## 1. `jig-protocol-engine-clients`

- **Its listener:** `Listen` at component.rs:95 (the link only on
  loopback, :70-77); sockets accepted and bound at :166-176; one
  skein-http server per connection (connection.rs, 132 lines); calls
  dispatched at component.rs:309-378, bodies and the bearer read at
  :380-451 (bearer :434, parsed in sign_in.rs:77); server-sent events in
  stream.rs and watches.rs:138-150, :421-495; the listener closed at
  closing.rs:253-285.
- **Routes** (routes.rs:18-42, `/jig/v1` hard-coded): GET `stream`; POST
  `sign-in/link`, `sign-in/deployment`, `sign-in/provider/<n>`
  (:106-120, which the design removes), `request`, `read`, `watch`,
  `unwatch`, `stream-gone`, `sign-out`. Everything else, a GET of the
  link's route included, is 404.
- **Guards:** none of browser.md, 12.2's: no Host check, no `Jig-Client`
  header, no cookie (bearer only), Content-Type never read, answers
  carry only Content-Type (connection.rs:84-87). It answers 413, 400,
  and 401 on the stream.
- **Size:** 10,147 lines, 2,973 production outside `translate/`; about
  550 to 650 move to `-http` (connection.rs 132, route dispatch about
  40, the component's io, HTTP and bodies about 200, events about 120,
  the listener about 40, the bearer and printed URL about 35, and
  `Limits.http`, `stall`, `connections`). The provider flow
  (closing.rs:169-252, about 85) gives way to `Mint`. The unit fixture
  drives the component with HTTP posts (src/tests/fixtures.rs:117-165,
  an 800-line file) and moves onto messages.
- **`Mint`:** a message in from the server's code, `Minted` and
  `MintRefused` out; reuses `provider_verdict`'s checks
  (closing.rs:218-251) and the `Keep` path (component.rs about
  470-490); adds the check that the provider is of kind application
  (`Settings.providers`, `ProviderKind::Application` exist), UTF-8 login
  and name, not ready while stopping, busy at the exchange limit.
  `SignedIn.token` is always in the answer's body today; a credential
  held by a cookie needs it absent.
- **Users of its listener:** `jig-test-service` (clients.rs:68-71 takes
  the listener from jig-protocol-config's clients.rs:7, :13-16; io
  tokens tagged `4<<60`, :13, :253-296; provider requests refused,
  :56-58; events routed at lib.rs:539-547; the listener checked at
  engine.rs:230-236 and stop.rs:32-56; HTTP limits in
  profile/small.rs:250-260 and the standard profile). Worlds:
  `tests/engine-clients` (1,639 lines; a raw HTTP peer sending Host
  `localhost`, peer.rs:47, which the allow-list would refuse; 34
  provider references); `tests/client-protocol` (3,026 lines; the
  service with the native shell, which already sends Host, Content-Type
  and Bearer, connections.rs:158-175); `tests/one-process` (signs in by
  the printed link, referee.rs:259-265). The repository downstream of
  jig: its service's `clients.rs` (498 lines; the listener at :51 and
  :96, the provider at :231-232) and 5 more files.

## 2. `jig-test-service` (`no_std`, lib.rs:2)

- Today `Clients` wraps the component (clients.rs:16-145) and routes
  what it asks for (`to_root`, :49-61). The link's URL bytes go into
  `links()` (lib.rs:284-292), which `jig-test` prints to stdout
  (host.rs:15, :53-57). It serves no HTTP, bundle or policy.
- Needed: a server child owning the listener, its settings (listener,
  certificate and key, bundle directory) members of `jig-test`'s own
  configuration (jig-protocol-config's `clients.listener` goes); skein-
  http per connection, `/jig` routed to `-http` with the link's address
  as the Host list; `/bundle/<digest>/` immutable, the page at every
  other GET and at the link's route (`Referrer-Policy: no-referrer`
  there), one root file; the reference policy with nosniff, COOP and
  CORP on every answer. The bundle read in `jig-test` (std) after
  skein's `examples/ui/server/src/bundle.rs:46-117`; the step-code side
  after its routes.rs (89 lines) and `Bundle` (117 lines). The start
  prints the URL from `Link { secret }`, which the component builds
  today (sign_in.rs:42-62).

## 3. skein's browser kit

- `skein-ui-web` (5,668 lines): fetch with method, path, headers, body,
  bound and same-origin credentials (fetch/web.rs:193-201), a streamed
  GET with headers; `transport::carry` (transport.rs:28-49); the
  prefixed small store; `history.show(path, replace)` (history/web.rs:
  43-53); the inbox, page, timer, title, restart and hidden-tab parts.
  Server-sent events are cut by skein-http, which skein's gate checks on
  wasm32.
- `skein-ui-bundle`: `--module --name --prefix --stylesheet…
  --pictures owner=dir --page --files --noscript --out`.
- The example: `examples/ui/shell-web/src/lib.rs` (275 lines with
  grow.rs) is the shell to copy; the server's routes.rs and bundle.rs
  carry the serving contract.
- `skein-fake-person-browser` (2,109 lines): `BrowserFace`; `Harness`
  with `Setup{server, args, listening, chromium, expected}`; `Script` and
  its steps (StopServer, StartServer, Warned, PressAt, Back, …); 4
  stories (panic, write refused, typing while patched, back and
  forward).
- skein's gate: `cargo check --target wasm32-unknown-unknown -p
  skein-lib -p skein-codec -p skein-http`; `cargo clippy --target
  wasm32-unknown-unknown -p skein-ui -p skein-ui-web -p
  skein-ui-example-view -p skein-ui-example-shell-web -- -D warnings`;
  after the fuzzy suite, `cargo build --target wasm32-unknown-unknown -p
  skein-ui-example-shell-web`, `cargo run -p skein-ui-bundle -- … --out
  target/ui-example-bundle`, `cargo build -p skein-ui-example-server`,
  `SKEIN_UI_EXAMPLE_BUNDLE=… cargo nextest run --workspace --profile
  browser`. The `browser` profile: the `browser_*` binaries, a 2-minute
  timeout, 1 thread. Chromium from `SKEIN_TEST_CHROMIUM`, else
  `chromium` on PATH (tests/ui-web/src/lib.rs:28-43). Pins:
  wasm-bindgen =0.2.126, js-sys and web-sys =0.3.103, `[profile.web]`
  at Cargo.toml:214-221.

## 4. What skein's harness cannot do yet

- **A page from a second origin: no.** The harness spawns only the
  server and Chromium (harness.rs:197-200); its io allows one accept,
  backlog 1 (:138-139); the per-person script gets no io (:32-41). The
  only page server is a test crate's (tests/browser/src/serve.rs, 250
  lines).
- **A framed page: no.** The first page is `http://{address}/`
  (harness.rs:558); the face's go reaches only `http://{address}{path}`
  (face.rs:265).
- **A policy's report as trouble: partly.** The kit reports it; the
  harness only tolerates trouble containing a `Setup.expected` substring
  for the whole run (harness.rs:336-345), cannot await it, and the face
  ignores trouble (face.rs:246).
- **jig's tier in general:** the harness reads the server's stderr only
  (harness.rs:183, :425), `jig-test` prints its link to stdout; it takes
  the printed line as host:port and appends `/` (:79-87, :558), which
  would corrupt a printed secret; a restart replaces the first argument
  with the address (:415), which breaks `jig-test serve CONFIG`; all
  people share one address (:540-548), so a second person opens a spent
  link.
- **The smallest skein change, about 250 to 300 lines:** (a) the printed
  line opened whole as a URL, from stdout or stderr, the latest line per
  person, restart arguments with an `{address}` placeholder; (b) extra
  origins: fixed pages served by the harness on extra listeners at a
  given host and port (serve.rs's server moved into the kit), and a step
  opening an absolute URL; (c) a step awaiting trouble with given words
  on the person's page, and a raw HTTP request with a given Host whose
  status is awaited (the rebound-Host story).

## 5. jig's gate

fmt, clippy, the default suite, the fuzzy suite: no browser profile, no
wasm32 build, no `web` profile, no dependency on skein-ui-web, the
bundler, the browser harness or the glue crates. The default filter
(not fuzzy, not process or live) would take `browser_*` binaries. Step
6 adds: the dependencies, the glue's pins and `[profile.web]`; `cargo
clippy --target wasm32-unknown-unknown -p jig-client-shell-web -p
jig-client-test-shell-web -p <each client crate> -- -D warnings`; the
module built and bundled (`--prefix /bundle/`, `jig.css` then the test
stylesheet, `--pictures jig=…`), `cargo build -p jig-test`, then
`JIG_TEST_BUNDLE=… cargo nextest run --workspace --profile browser`;
`[profile.browser]` (a 60 s global timeout, 1 thread); `not
binary(/^browser_/)` in the default and fuzzy profiles; a third suite in
workflow.md, section 2. jig's client crates are `no_std` and use no std
time, fs or env; their wasm32 build was not tried.

## 6. Order

- Step 5 after step 3 (both rewrite component.rs, watches.rs and
  reads.rs). Within it: `Mint` first; the component's message boundary
  before the guards; the listener into the server before the bundle.
- Step 6 after steps 1 and 4 (the names, `jig-client-protocol-http`,
  `jig-client-view` on the public client, the small store without
  places); the reference UI's codec (place to path) is new in step 6.
  Within it: skein's harness change before any jig browser test; the
  wasm32 build before the adapter.
- Sizes: step 5 about 1,500 production lines; step 6 about 2,300.
