# forge-setup: later

| Item | Design | When |
|---|---|---|
| GitHub's setup in its three modes | setup.md 15 | GitHub's build |
| The binaries' digests | setup.md 15 | a deployment that installs binaries it did not build or download |
| Webhooks from another host, over TLS | setup.md 15; forge.md 5 | skein's TLS server side |
| CI's isolation (an owned runner's jobs as another user) | setup.md 15 | before an owned forge runs a stranger's pull request |
| Provisioning at adoption | setup.md 15; domain/connectors.md 11 | a repository in a person's namespace |
| The forge as people's sign-in provider | setup.md 15; credentials.md 6 | a plan of its own |
| Reconcile over TLS (managed mode, a remote forge) | setup.md 5, 15 (the designer's answer of 2026-10-16) | a managed forge that is not on loopback |
| The lock and the exec on skein's shell | setup.md 15 (the designer's answer of 2026-10-16) | skein's shell offering a lock and an exec |
| temper-engine-domain-forge-issues' `sha2` moved onto skein-lib's `sha256` | lib.md 6.2 | temper's next plan that touches the issues domain |
| Repins of smith, jig and hearth onto skein's `sha256` | decisions.md 3 | each repository's next repin |
| The fake forge's deliveries to the org hook reconcile made, in a world (forge.md 9; testing.md 4.2) | decisions.md 19 | a world that runs reconcile and the engine together |
| The protocol world's late, reordered and lost deliveries, its stories run with hints and without (forge.md 9, 10) | decisions.md 19; the connector world's hinted stories (temper 06) stand for them | a fake that delays, swaps and drops its deliveries |
| A change's pull request in the keep's working set, its CI read on `poll` to `poll_max` and a read waking the change (domain/forge.md 5.1, 5.2): a change names its branch and base alone (temper-engine-domain `route/forge_route.rs`:1582), so with no hint its CI is read only at its `stall`; and the shipped `stall`, 30 s from a world fixture (temper-service `src/limits.rs`:154, :323), holds a change whose CI runs longer, hooks or not | domain/forge.md 5.1, 5.2, 8.4; the designer's answer to temper 11.2's Q2, 2026-10-18 | before a box runs a repository whose CI takes longer than 30 s: the supervisor places it |
