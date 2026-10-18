# The web's architecture

Provisional, 2026-10-05, revised 2026-10-18. What stays true of
temper's browser client: a later shell of temper's client (../client.md),
on skein's browser parts. What its pages do is the client's (../client.md;
`ux/forge.md`), with the mockups' look (`mockups/`), which this document
takes as given; how a view runs in a browser is skein's (skein's
`ui-web.md`). What is still open is section 5.

## 1. In one page

- **The browser is a later shell of the same client.** temper's client
  root, its view and the forge's family (../client.md, sections 3 to 5),
  compiled to WebAssembly under a browser shell of temper's,
  `temper-client-shell-web`, on skein's browser parts (`skein-ui-web`;
  skein's `ui-web.md`), with jig's default HTTP mapping's client end over
  fetch (`jig-client-protocol-http`). temper writes no web wire and no
  web protocol of its own: the client's records cross as jig's wire
  carries them, binary records from a schema per owner, never JSON
  (jig's `protocol/client.md`, section 3).
- **No framework.** The client and its view are step code under the
  subset in a browser too (skein's `ui.md`, section 1; section 3 below).
- **The view tree is the fake DOM.** Below the browser, every world
  reads skein-ui's tree by role, name and text through skein's fake
  person (skein's `ui.md`, 17.1): no browser, no WebAssembly and no fake
  DOM library.
- **Served by temper's server, from one origin.** `temper-service` serves
  the page and the bundle on its clients' listener (skein's `ui-web.md`,
  4.4), beside jig's clients' component under `/jig` (../client.md,
  section 8), with jig's example guards and the page's policy, which is
  temper's: a strict content security policy, no frames, no CORS (jig's
  `protocol/browser.md`, section 12).
- **A real browser at the top, driven from Rust,** by `skein-browser`
  (section 4).

## 2. What temper brings to the browser

- **Its crates as they are.** The client root, the forge's side, the
  view and `temper-client-protocol-wire` are step crates under the
  subset, which build for the host and for wasm32 alike; the terminal is
  not part of the bundle.
- **Its look:** the mockups' (`mockups/style.css`), as temper's style
  tokens and a stylesheet over skein's (skein's `ui-web.md`, 3.2 and
  section 12).
- **Its pages,** the client's (../client.md, 3.5), each at an address of
  its own (jig's `client/domain/pages.md`, section 2; skein's
  `ui-web.md`, section 9); and the forge's parts, from `ux/forge.md`,
  as the forge serves their documents.
- **Signing in:** in standalone, the browser opens the link `temper
  serve` prints. Its `GET` spends nothing: temper's page answers it,
  with `Referrer-Policy: no-referrer`; the shell reads the secret from
  the address, rewrites the address at once, and signs the client in by
  the link (`SignIn { with: Link(secret) }`), whose exchange is the link
  route's `POST`, with the browser header. The engine end answers it
  with the credential in the mapping's cookie, which the client never
  sees; after a reload the client signs in with `Transport` (jig's
  `protocol/browser.md`, section 11; jig's `client/domain/client.md`,
  section 12). The forge as people's sign-in provider is temper's own
  routes, later (../client.md, section 11).
- **The toolchain:** the standard library for `wasm32-unknown-unknown`,
  skein's bundler and the glue it names (skein's `ui-web.md`, section 4),
  and Chromium for the browser tier.

## 3. Why no framework

The strongest candidate is Dioxus, whose native DOM makes it the best of
Rust's frameworks for testing without a browser; Leptos and Yew share
the first two objections.

- **Its model is the opposite of skein's `programming-model.md`:**
  closures, signals with interior mutability, hooks, async tasks and
  procedural macros, which pull state out of the domain, where worlds
  see it.
- **It brings its own scheduler,** a second loop in every world.
- **Its fake DOM does not come apart from it:** testing with it means
  building with it.

The choice stays reversible: a framework could render the same tree,
and the domain, its worlds and its stories would not change.

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
  is skein's browser parts' to test;
  temper's tier checks its own bundle, its pages, signing in, and jig's
  attack stories that need a browser, a cross-site request and a framed
  page (jig's `protocol/browser.md`, section 15).
- **A suite of its own,** outside the focused and fuzzy suites: it needs
  Chromium and does not replay (../testing.md, section 9).

## 5. Open questions

- **When temper builds its browser shell.** Default: temper's only
  client is the terminal (../client.md).
- **The browser tier's budget,** set with skein's browser tier's (skein's
  `ui-web.md`, 14.1). Default: a suite of its own, run by the gate.
- **Other browsers** than Chromium are `skein-browser`'s question
  (skein's `browser.md`).
