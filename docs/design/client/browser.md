# The browser

Provisional, 2026-10-05, revised 2026-10-18. temper's browser client: a
later shell of temper's client (`README.md`), on skein's browser parts.
What its pages do is the client's (`README.md`; `forge.md`), with the
mockups' look (`mockups/`, reference material), which this document
takes as given; how a view runs in a browser is skein's (skein's
`ui-web.md`), and how a client of jig's does is jig's (jig's
`protocol/browser.md`). What is still open is section 5.

## 1. In one page

- **The browser is a later shell of the same client.** temper's client
  root, its view and the forge's family (`README.md`, sections 3 to 5),
  compiled to WebAssembly under a browser shell of temper's,
  `temper-client-shell-web`, on skein's browser parts (`skein-ui-web`;
  skein's `ui-web.md`) and jig's client adapter and sign-in part
  (`jig-client-shell-web`; jig's `protocol/browser.md`, 5.1), with
  neither jig's look nor its reference UI. The adapter carries jig's
  default HTTP mapping over fetch (`jig-client-protocol-http`). temper
  writes no web wire and no web protocol of its own: the client's
  records cross as jig's wire carries them, binary records from a schema
  per owner, never JSON (jig's `protocol/client.md`, section 3).
- **No framework.** The client and its view are step code under the
  subset in a browser too (skein's `ui.md`, section 1).
- **The view tree is the fake DOM.** Below the browser, every world
  reads skein-ui's tree by role, name and text through skein's fake
  person (skein's `ui.md`, 17.1): no browser, no WebAssembly and no fake
  DOM library.
- **Served by temper's server, from one origin.** `temper-service` serves
  the page and the bundle on its clients' listener (skein's `ui-web.md`,
  4.4), beside jig's clients' component under `/jig` (`README.md`,
  section 8), with jig's example guards and the page's policy, which is
  temper's: a strict content security policy, no frames, no CORS (jig's
  `protocol/browser.md`, 12.1 and 12.2).
- **A real browser at the top, driven from Rust,** by `skein-browser`
  (section 4).

## 2. What temper brings to the browser

- **Its crates as they are.** The client root, the forge's side, the
  view and `temper-client-protocol-wire` are step crates under the
  subset, which build for the host and for wasm32 alike; the terminal is
  not part of the bundle.
- **Its look:** after the mockups (`mockups/style.css`), as temper's
  style tokens and a stylesheet over skein's (skein's `ui-web.md`, 3.2
  and section 12).
- **Its pages,** the client's (`README.md`, 3.5), each at an address of
  its own (jig's `client/domain/pages.md`, section 2; skein's
  `ui-web.md`, section 9); and the forge's parts, from `forge.md`, as
  the forge's side reads them.
- **Signing in:** in standalone, the browser opens the link `temper
  serve` prints, and jig's sign-in part signs the client in by it (jig's
  `protocol/browser.md`, section 11). temper's page answers the link
  route's `GET`, with `Referrer-Policy: no-referrer` (`README.md`,
  section 8); the credential is the mapping's cookie, which the client
  never sees, and after a reload the client signs in with `Transport`
  (jig's `client/domain/client.md`, section 12). The forge as people's
  sign-in provider is temper's own routes (`README.md`, section 11).
- **The toolchain:** the standard library for `wasm32-unknown-unknown`,
  skein's bundler and the glue it names (skein's `ui-web.md`, section 4),
  and Chromium for the browser tier.

## 4. The browser tier

- **The driver is skein's.** `skein-browser` (skein's `browser.md`) runs
  headless Chromium as a child and drives it over the DevTools protocol
  on a pipe, from the loop beside the engine.
- **The person's browser face is skein's** (`skein-fake-person-browser`;
  skein's `ui-web.md`, 14.2): the same queries and acts as on the tree,
  made over Chromium's accessibility tree. temper's stories of
  `tests/client`, written in the person's terms, run there against
  temper's engine in the same loop.
- **What only a browser shows** (focus, caret and scroll kept across a
  patch, session storage across a reload, the bundle under its policy)
  is skein's browser parts' to test; temper's tier checks its own
  bundle, its pages, signing in, and jig's attack stories that need a
  browser, a cross-site request and a framed page (jig's
  `protocol/browser.md`, section 15).
- **A suite of its own,** outside the focused and fuzzy suites: it needs
  Chromium and does not replay (`../testing.md`, section 2).

## 5. Open questions

- **The browser tier's budget,** set with skein's browser tier's (skein's
  `ui-web.md`, 14.1). Default: a suite of its own, run by the gate.
- **Other browsers** than Chromium are `skein-browser`'s question
  (skein's `browser.md`).
