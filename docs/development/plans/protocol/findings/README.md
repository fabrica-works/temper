# protocol: findings

The planner's code audit (development.md, 5.4), 2026-10-14: seven
read-only sub-agents, one per area, at temper `cc2697cf` and jig
`d91a89c4`, smith `dfc0fe57`, skein `b5d3a2c3`. Each file is a report as
written, with paths and lines; landed code wins where it and a report
differ. This page maps each finding to what the plan does about it.

| File | Area |
|---|---|
| `jig-service.md` | jig's reference service, binary, process and live tiers, one-process world |
| `jig-components.md` | jig's store, channel, accounts and clients' components, as temper's application parts meet them |
| `jig-client.md` | jig's client domain, view, link, native shell, fake person and client world kit |
| `skein-smith.md` | skein's codec generator, HTTP, JSON, TLS, kv, shell, io and test kit; smith's agent shell, host half, codecs and sign-in |
| `temper-engine.md` | temper's engine root, forge connector and client, `temper-engine-smith`, the worlds, the large files |
| `temper-worker.md` | temper's worker root and checkout against git.md and channel.md; skein-io's processes and files |
| `temper-misc.md` | the retiring web crates, the Python conformance tool, the suites, crate kinds, pins, `STATUS.md` |

## Findings and what the plan does

| Finding | Where | Acted on by |
|---|---|---|
| The root drops a rendered error for a call that never decoded, so the run waits forever | temper-engine.md, 2(d) | 01.1 |
| `Parts` has no preparation; jig's core keeps no failure detail for an attempt | temper-engine.md, 2(c), Finding 5 | the design fix (a fact of the attempt); 01.2 |
| The worker's saved work and a failed preparation name repositories by place, not tag | temper-worker.md, Findings 1, 2 | 01.3 |
| No `Read::Locate`; the adoption needs the repository's id up front; the fake forge has no ids | temper-engine.md, 2(a), Findings 2, 12 | 02.1, 02.3 |
| The keyed adoption is not built; jig's application request flow is | temper-engine.md, 2(b), Finding 3; jig-components.md, 5 | 02.2, 02.3 |
| No family keeps a keyed adoption's outcome, which jig asks a connector to keep durably | temper-engine.md, Finding 4 | the design fix (family `Application`); 02.4, 04.4 |
| An unconfigured forge refused as `Forbidden`, not "no such forge" | temper-engine.md, Finding 7 | 02.3 |
| `temper-engine-smith::start` leaves the charter's resume rule to the system world | temper-engine.md, Finding 11 | 08.4 (`charter`) |
| The forge client's `Error` variants are in another order than forge.md, 3.4's | temper-engine.md, Finding 14 | 04.3 numbers them explicitly |
| Adding a jig, smith or skein crate not in the lock moves the pins | temper-misc.md, 6; the planner's check | code-organization.md, section 3; decision 5 |
| skein-io's file layer renames within one directory; opened directories come back as tokens | temper-worker.md, Findings 3, 4, 5 | decision 7; 07.3 |
| skein's fake machine runs no application program | temper-worker.md, Finding 6; skein-smith.md, D | the design fix; 17.1 |
| jig ships no generic store driver for worlds; the component still asks for transcript files in memory mode | jig-components.md, Discrepancy 5 | decision 14; 18.1 |
| The store's sizes are checked by a function the caller runs; the restore's cursor holds the tagged key | jig-components.md, Discrepancies 3, 4 | 08.2 |
| An application credential's value is looked up by the routing; a missing one drops its grant silently | jig-components.md, Discrepancy 6 | 08.3 (every identity is filled at start; a test that each repository's grant carries a value) |
| jig's reference service sends and drops empty parts; its `service.components` gates components; it steps the restart before the credential | jig-service.md, section 1, Discrepancies 4, 5 | 08.2, 08.3 (a question with its default) |
| jig's binary has `serve` and `agent` only, prints problems as one `Debug` line, opens worker roots without making them | jig-service.md, Discrepancies 2, 3, 6 | 10.1, 10.3; 16.4, 16.5 |
| jig's live script reads the login with Python, and its budgets are 5 completions and 5 minutes | jig-service.md, Discrepancy 9 | decision 11; 15.4 |
| jig's process tests use the system's temporary directory | jig-service.md, Discrepancy 9 | decision 13; 10.4 |
| `Identity`'s login and author print in its `Debug` | skein-smith.md, F | the design fix |
| smith's sign-in prints the visit on stderr and exits 2 or 3 | skein-smith.md, E | 16.5 |
| No `ByteRing` or capped growing buffer in skein's lib | temper-worker.md, 6; skein-smith.md | 07.3, 13.2 (modules of their own); later.md |
| The client world kit is `jig-client-world`'s traits; the application reference is ops's | jig-client.md, 5, Discrepancy 6 | decision 16; 11.3 |
| jig's native client is world code, panics on sign-in paths, has no family hello | jig-client.md, Discrepancies 1 to 3 | decision 17; 12.1 |
| No jig view builds a check; the fake person never toggles | jig-client.md, Discrepancy 13 | 15.1 (bindings take press and toggle) |
| The navigation's target is the frame's project, which neither reference reads | jig-client.md, Discrepancy 10 | 15.1 |
| jig-view's `Cell` role and statuses with no text | jig-client.md, 6, Discrepancy 12 | the design fix; 16.1 |
| The sign-in page offers a button only for a registered provider; temper registers none | jig-client.md, Discrepancy 15 | 16.3 (the shell's line names the link) |
| Grant patterns have no slot for the forge's words | jig-client.md, Discrepancy 14 | later.md |
| `jig-shell-native` gives no reason for a refused directory | jig-client.md, Discrepancy 4 | 16.4 (a question with its default); later.md |
| The base-branch creation and pushes without an expected head are still built | temper-worker.md, Findings 7, 8 | later.md (not planned) |
| The worker world never plays `CancelIo` | temper-worker.md, Finding 13 | 07.4's machine world covers the component's cancel; 09.2's routing tests |
