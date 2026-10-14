# The web's architecture

Provisional, 2026-10-05, revised 2026-10-14. What stays true of
temper's browser client: a later shell of temper's client
(`../client.md`), which waits for jig's browser shell. What the web
does is `ux/`, which this document takes as given; how jig's client
runs in a browser is jig's. What is still open is section 5.

## 1. In one page

- **The browser is a later shell of the same client.** temper's client
  root, view root and the forge's family (`../client.md`, sections 3
  to 5), compiled to WebAssembly under jig's browser shell,
  `jig-shell-web` (jig's `protocol/client.md`, section 12; jig's
  `client/domain/view.md`, 11 to 13), once jig builds it. temper writes
  no browser shell, no web wire and no web protocol of its own: the
  client's records cross as jig's wire carries them, binary records
  from a schema per owner, never JSON (jig's `protocol/client.md`,
  section 3).
- **No framework.** The client is step code under the subset in a
  browser too (jig's `client/domain/view.md`, section 1; section 3
  below).
- **The view tree is the fake DOM.** Below the browser, every world
  reads the view tree by role, name and text through jig's fake person
  (jig's `client/domain/testing.md`, section 3): no browser, no
  WebAssembly and no fake DOM library.
- **Served by the engine from its one origin,** under a strict content
  security policy, with no CORS (jig's `protocol/client.md`,
  section 12).
- **A real browser at the top, driven from Rust,** by `skein-browser`
  (section 4).

## 2. What temper brings to the browser

- **Its crates as they are.** The roots, the forge's side and view, and
  `temper-protocol-client-wire` are step crates under the subset, which
  build for the host and for wasm32 alike; the terminal shell is not
  part of the bundle.
- **Its look:** the forge's style tokens, in the stylesheet jig ships,
  extended (jig's `client/domain/view.md`, section 13). The mockups'
  stylesheet (`mockups/`) is where temper's starts.
- **Its pages,** as `ux/` intends them: jig's pages
  (jig's `client/domain/pages.md`) and the forge family's, from
  `ux/forge.md`, as the forge serves their documents.
- **Signing in:** in standalone, the browser opens the link the engine
  prints (jig's `protocol/client.md`, section 8); the forge as people's
  sign-in provider is a provider component of temper's, later
  (`../client.md`, section 11).
- **The toolchain:** the standard library for `wasm32-unknown-unknown`,
  the glue generator jig's browser shell names, and Chromium for the
  browser tier.

## 3. Why no framework

The candidate was Dioxus: of Rust's frameworks, its native DOM made it
the strongest for testing without a browser; Leptos and Yew share the
first two objections.

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
- **The person's browser face is jig's** (jig's
  `client/domain/testing.md`, section 3): the same queries and acts as
  on the tree, made over Chromium's accessibility tree. temper's
  stories of `tests/client`, written in the person's terms, run there
  against temper's engine in the same loop.
- **What only a browser shows** (focus, caret and scroll kept across a
  patch, session storage across a reload, the bundle under its policy)
  is jig's shell's to test; temper's tier checks its own bundle, its
  forge's pages and signing in.
- **A suite of its own,** outside the focused and fuzzy suites: it needs
  Chromium and does not replay (`../testing.md`, section 2).

## 5. Open questions

- **When jig builds `jig-shell-web`.** Default: temper's only client is
  the terminal (`../client.md`).
- **The browser tier's budget,** set with jig's when its first tests are
  built (jig's `client/domain/testing.md`, section 10). Default: a
  suite of its own, run by the gate.
- **Other browsers** than Chromium are `skein-browser`'s question
  (skein's `browser.md`).
