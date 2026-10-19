# Survey 04: temper's forge setup, for skein-forge-setup

Read only. Repos at origin/main. No `skein-forge*` crate exists in skein/crates yet (design is ahead of code).

```
$ ls skein/crates | rg forge                                   (no output)
$ rg -c '#\[test\]' temper/crates/temper-forge-setup{,-shell} temper/tests/forge-setup
$ wc -l <each src file>; rg -n '#\[cfg\(test\)\]' temper/crates/temper-forge-setup/src
$ rg -n 'serde' temper/crates/temper-forge-setup{,-shell} temper/tests/forge-setup
$ rg -n 'temper_forge_setup|temper-forge-setup' --glob '*.rs' --glob 'Cargo.toml' temper
$ rg -n 'temper|bot|user_agent' temper/crates/temper-forge-setup/src/reconcile*
$ sed -n 180,215p temper/crates/temper-forge-forgejo/src/client/orgs.rs
$ sed -n 60,110p temper/crates/temper-forge-forgejo/src/delivery.rs
(+ search_graph / get_code_snippet on project srv-data-git-runner-src-temper, names re-checked with rg)
```

## (a) The two crates

`temper-forge-setup` (no_std outside tests). Deps: temper: `temper-forge-forgejo`; jig: `jig-protocol-config`;
skein: `skein-lib`, `skein-json`; smith: none; external: none. Production 2853 lines, tests 1756, 98 `#[test]`.

| file | prod | test | #[test] | verdict |
|---|---|---|---|---|
| reconcile.rs | 340 | in tests/ | | MOVE (machine: Need, Event, Reply, Failure, Stage, Why, step) |
| reconcile/hooks.rs | 200 | | | MOVE (`hook_url` loopback rule) |
| reconcile/orgs.rs | 175 | | | MOVE (team, members) |
| reconcile/tokens.rs | 216 | | | MOVE (API/git token keep or remake) |
| limits.rs | 65 | 31 | 3 | SPLIT: bound check moves; `setup.*` names stay |
| check.rs | 162 | | | STAY (temper's person and limit-name checks) |
| problem.rs | 113 | | | STAY (`application.setup` schema, jig_protocol_config) |
| setup.rs | 283 | | | STAY (Entry, Mode, Account; feeds the machine its given values) |
| place.rs 71, state.rs 103 | 174 | | | STAY (File places; host-owned) |
| spec.rs 187, ini.rs 100, runner.rs 47 | 334 | | | STAY (owned Forgejo preparation, forge-setup.md 9) |
| prepare.rs 550, upgrade.rs 58, run.rs 79, step.rs 82, lib.rs 22 | 791 | | | STAY (box steps, migrations, dumps) |
| tests/ (13 files) | | 1756 | 98 | reconcile 13, members 7, read 5, steps 4 move with the machine; prepare 13, spec 13, upgrade 19, run 13, files 4, golden 4, limits 3 stay; fixtures 383 split |

`temper-forge-setup-shell` (std). Deps: temper: `temper-forge-setup`, `temper-forge-forgejo`; skein: `skein-io`, `skein-http`,
`skein-lib`, `skein-shell`, `skein-json`; dev: `temper-service`, `temper-engine-protocol-forge`, `jig-protocol-config`; smith: none;
external: none. Production 1116 (command 215, exchange 229, files 73, lib 12, prepare 88, process 55, reconcile 395, run 49).
Tests 504, 24 `#[test]` (boundary 2, commands 3, exchange 3, files 3, locks 2, reconcile 11, support 0). ALL STAYS: it is the host
(places, lock, wait, backoff, exit status) and uses a hand-rolled `Peer` on a `TcpListener`, not a fake forge.

Blockers (moving part depends on non-skein code), symbols used:
- `temper_forge_forgejo` (-> skein-forge, per forge.md): `request::{Request, Page, HookSpec}`,
  `document::{Document, Row, User, Access, TeamSpec, Units}`, `delivery::{SUBSCRIBED, hook_target}`. `pin::*` is not used by the machine.
- `place::File` (temper): `Need::Read(File)` and `Write{file}`; variants used AdminToken, ApiToken, GitToken, User, WebhookSecret
  (enum has 13; `tokens.rs name()` matches exhaustively). Needs a kit `Item` enum.
- `setup::{Entry, Mode::Owned.person.login, Account}`: `Reconcile::new(&Entry, listener)` and `Orgs::new` read them. Needs the design's given-values struct.
- jig: none in the machine. `jig_protocol_config` is only in check.rs and problem.rs (stay). smith: none. temper: only the two above.

## (b) Temper-specific, to become given values
- `reconcile.rs`: `b"temper-admin"` (in `stop`), `b"temper:bot"` (marker check in `id()`, `full_name` in `draw()`).
- `reconcile/tokens.rs`: `b"temper-api"`, `b"temper-git"`, and both scope lists.
- `reconcile/orgs.rs`: team spec (write, 4 units, `includes_all_repositories`) fixed; org owner and make-org flag come from `Mode::Owned`.
- `spec.rs`: `ADMIN = b"temper-admin"`, `b"temper-admin@example.invalid"`, `b"temper-runner"`; `check.rs`: `temper-admin` person check, "no limit of temper's has this name".
- `limits.rs`: names `setup.*`; `problem.rs`: "application.limits.overrides."; `step.rs`: subcommand names. All stay.
- Shell: `Refused::line()` ("temper forge reconcile: forge N: ...", "has no temper:bot marker"), `Settings.user_agent`,
  `File` mapping in `bound()`/`file_path()`, `token <value>` header in `exchange.rs`. All stay (host).
- Given values still missing from the code as parameters: word, token names, admin name, bot login/email, org list, limits as one struct. Limits 8/120 s/100 ms/5 s are in `limits.rs`, not in the machine.

## (c) Audit against forge-setup.md
Verified by reading code: step order (admin, bot, tokens, bot id, teams, hooks); `MAX_OUT` 1; events and needs as in 4.2;
password is 32 drawn bytes as 64 hex digits; bot marker check; token kept only if `GET /user` answers as the bot, else deleted and remade;
page cap read once (zero is `Unexpected`); bot id written in decimal; team made or edited, bot added; hook: lowest id kept, others deleted,
kept one patched, created if none; hook URL only for a loopback listener, else `Done` without hook (`hook_url` tested v4 and v6);
`CreateOrg` posts `/api/v1/admin/users/<owner>/orgs` with `username` and `visibility: private`; failure carries forge, step, subject, why.

Differences: (1) given values hardcoded or derived from `Mode` (see b); (2) `File` where the design says items; (3) `BotIsAdmin` is
tested before `Unmarked` (design lists `Unmarked` first; matters only when both apply); (4) the org-count bound lives in the config reader, not the machine;
(5) the machine takes `&Entry`, not a free-standing given-values struct. Not verified: re-listing the same hook page after a deletion
(no test drives that order); behavior of a duplicate-token name race. In-crate reconcile tests (13) lack revoked-token, hook-patch and
out-of-order-event cases; the world crate covers the first two.

## (d) serde
`rg serde` in `temper-forge-setup`, `-shell` and the reconcile world: 0 hits per file. No serde, no serde_json; JSON goes through skein-json.
Elsewhere (not part of the move): `crates/temper/Cargo.toml:60` `serde_json` is a dev-dependency; serde lines in tests/forge.rs 2 and tests/support/owned.rs 35 are setup-related.

## (e) Users (use-site counts are `rg -c` of `temper_forge_setup`, which also matches `_shell`)
- Cargo: temper `Cargo.toml` members lines 21 and 22, deps 60 and 61; `temper-box/Cargo.toml:10`; `crates/temper/Cargo.toml:17,19`; shell `Cargo.toml:10`; world `Cargo.toml:14`.
- `temper-box`: src/host.rs, machine.rs; tests/fixtures.rs, tests/boundary.rs (use `Step` only).
- `crates/temper`: src/forge.rs 6, setup.rs 1, box_command.rs 1; tests/forge.rs 8, process.rs 2, support/owned.rs 9.
- shell crate: uses the machine throughout (reconcile.rs). World crate: exchange 1, fixtures 6, world 1, fuzzy 1, reconcile 5.
- Binaries: `temper` (reconcile and box commands), `temper-box`.

## (f) Tests
In-crate: 98 + 24 (above). World: `temper/tests/forge-setup/reconcile`, crate `temper-forge-setup-reconcile-world`:
src exchange 181, fixtures 226, lib 5, world 159; tests/reconcile.rs 275 with 21 `#[test]`, tests/fuzzy_reconcile.rs 21 with 1.
Deps: skein-lib, skein-http, jig-protocol-config, temper-forge-forgejo, temper-forge-setup, `temper-fake-forge-protocol`,
`temper-engine-protocol-forge-world`, `temper-codecs-world`. It uses the temper fake forge (`Service`, `Tally`, `setup::Store`, `Config`, `Faults`, `Route`).

Other: `crates/temper/tests/process.rs` and `support/{owned,forgejo}.rs` run a real Forgejo (`temper-throwaway-forgejo`), not the fake. `tests/forge.rs` uses scripted programs.
temper/testing has temper-fake-forge-domain, temper-fake-forge-protocol, temper-fake-git, temper-throwaway-forgejo.

Move with the machine: in-crate reconcile, members, read, steps tests (29) and the reconcile fixtures. Stay: all shell tests (24), prepare, spec, upgrade, run, files, golden, limits tests.
ORDERING: the world crate (22 tests) needs the fake forge, which arrives in skein's later fakes session. The world cannot move in this
session; keep it in temper over `temper-fake-forge-protocol`, re-pointed at the skein machine, and move it after the fakes session. The in-crate machine tests need no fake.
