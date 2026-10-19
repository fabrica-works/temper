# Design trace: forge-setup.md 2-4, 6-8 and temper's protocol/setup.md (S04, T04, S06.3)

Code read at temper 27995fb75 (no change to the setup crates, the shell or the world since the
outline's base 62d50e0b8). forge-setup.md exists only on skein's local HEAD d120820, not on
origin/main b4cc6eb. No skein-forge* crate exists: "as it moves" means temper's code.

```
$ git -C temper log --oneline 62d50e0b8..HEAD -- crates/temper-forge-setup{,-shell} tests/forge-setup   (empty)
$ cat tmp/planning/forge-kit-outline.md findings/survey-04-setup.md; sed -n 1,420p skein/docs/design/forge-setup.md
$ sed -n 1,230p temper/docs/design/protocol/setup.md; sed -n 474,535p ...; sed -n 714,870p ...
$ cat temper/crates/temper-forge-setup/src/{reconcile.rs,reconcile/{hooks,orgs,tokens}.rs,limits.rs}
$ rg -n -A1 '#\[test\]' temper/crates/temper-forge-setup/src/tests/{reconcile,members,read,steps,limits}.rs
$ rg -n -A1 '#\[test\]' temper/tests/forge-setup/reconcile/tests/*.rs; sed -n 148,226p .../src/fixtures.rs
$ sed -n 205,300p temper/crates/temper-forge-forgejo/src/client/setup.rs   (request bodies)
$ rg -w -F -- "$fact" temper/tools/forgejo-conformance/facts.txt   (36 facts of forge-setup.md 8: all present)
$ rg -n 'must_change' skein/docs/design temper/docs/design/protocol/setup.md   (no hit)
$ rg -l temper_forge_forgejo temper/crates/temper-forge-setup{,-shell}/src   (staying prepare.rs, run.rs use the pin)
```

## 1. Gaps

### Kind (b): the design is silent or ambiguous (a design question)

- **B1 marker match** (4.3 step 2: "a full name without `<word>:bot`"). Code: substring (`bytes::find`),
  so word `b` accepts a bot named `ab:bot`, against 4.4 "another's bot is `Unmarked`". Options: (1) substring
  (as now); (2) full name equals `<word>:bot`; (3) the marker at the start or after a byte outside
  `[a-z0-9-]`. Recommend 3, built in 04.2 (the move changes nothing).
- **B2 BotIsAdmin or Unmarked first when both apply.** 4.3 step 2 lists Unmarked first, 4.4's list has BotIsAdmin
  first, code tests `is_admin` first. Options: (1) Unmarked first (the login is a person's account, and "demote
  the bot" misleads); (2) BotIsAdmin first. Recommend 1, with a test that sets both; 04.2.
- **B3 empty `orgs`.** 4.1 says "at most the limit's"; temper's setup.md 8 says "at least one". Code panics
  (`Orgs::name` and `Hooks::name` `.expect("current org")`). Options: (1) the kit's check refuses zero, naming
  `orgs`; (2) zero allowed, `Done` after step 4. Recommend 1 (what temper's reader does).
- **B4 what the kit's check owns, and its refusal's place** (6.1: "a refusal names the value's place"). Silent on
  which values are checked, and on the place's type. Needed: the word (1-16 `[a-z0-9-]`, forge.md 6); bot login,
  email and org names (shapes today in temper's `setup.rs` `login()`/`email()`); token names non-empty AND distinct
  (equal names: the git step's listing deletes the API token, a run that never converges); orgs 1..=`orgs`; owner
  given iff orgs are made; limits each >= 1. Also: a zero limit names the limit, and when both relations break
  (`backoff > backoff_max` and `backoff_max >= deadline`) which pair (code: the first). Options: (1) `check(&Given)
  -> Result<(), Refused { place: Place, .. }>`, `Place` a field enum the owner maps to its paths (temper maps to
  `setup.*` and `application.setup[..]`); (2) the owner checks all but the limits; (3) strings. Recommend 1, pair
  order as the relation is written. Moved with 04.1.
- **B5 "whether an absent org is made" beside "the orgs' owner"** (4.1): two values, one meaningless
  combination (made, no owner). Options: (1) one `Option<owner>`, Some = made; (2) both, with a refusal.
  Recommend 1 (setup.md 5's table says both come from `mode.owned.person.login`).
- **B6 an empty item.** 4.3 step 1 says "absent or empty" for the admin's token only. Code: a kept API/git token
  `Some(empty)` is sent in `GET /user` (a wasted request, then 401 and a remake: converges); a webhook secret
  `Some(empty)` is kept and put in the hook (an empty HMAC key). Options: (1) empty is none for every item;
  (2) the admin's only. Recommend 1; a behaviour change, 04.2.
- **B7 a kept token answered 403.** Step 3: "Otherwise (none kept, a 401, or another user's answer) it is made
  again". Code remakes on 401 or another user, and stops `Refused(403)` otherwise. facts
  `setup-a-git-token-of-write-repository-alone-is-refused-user`: a scope too small answers 403, so such a
  kept git token stops every run. Options: (1) as now; (2) 403 and 401 remake, 5xx and Down stop. Recommend 2.
  Scopes of a kept token that answers as the bot are never compared (listing shows them): say so or compare.
- **B8 where temper's mapping lives and is tested.** setup.md 11: the shell "maps the setup onto the machine's
  values"; 13.1 lists it among step tests. Options: (1) a pure `given(&Entry)` in `temper-forge-setup` (a new edge
  to `skein-forge-setup`); (2) in the shell, tested by its ordinary tests. Recommend 2, reword 13.1.
- **B9 the graph of setup.md 11 lacks an edge.** `temper-forge-setup` checks the `setup.` limits for every command
  (11, bullet 1), and the relation moves to the kit, but 11 draws `temper-forge-setup` over skein-forge,
  jig-protocol-config, skein-lib, skein-json only. Options: (1) add `skein-forge-setup`; (2) keep a second copy
  of the relation in temper. Recommend 1.
- **B10 `must_change_password: false`** (the created bot, an observable key; the in-crate test asserts it,
  tests/reconcile.rs:40) is in neither 4.3 step 2 nor section 8. The facts exist
  (`setup-a-user-made-to-change-its-password-is-refused-the-api-under-basic-auth`, `...-under-its-token-until-
  the-admin-clears-the-flag`). Recommend: say it in step 2 and name both facts in 8. No code work.

### Kind (a): the design is clear (add to the increment)

- **04.1 (the move).**
  - Hardcoded names become given values: `b"temper-admin"` (`stop`, Admin subject), `b"temper:bot"` (`id()` and
    `draw()` full name: `<word>:bot`), `b"temper-api"`/`b"temper-git"` and the two scope lists (scopes stay fixed
    in the kit, 4.3). `Need::Read(File)`/`Write{file}` become `Read(item)`/`Write{item, bytes}`; the items
    are admin token, API token, git token, bot id, webhook secret. `place::File` stays temper's (13 variants).
  - The neutral `Limits{orgs, deadline, backoff, backoff_max}` and its check move; temper's `setup.*` names, `apply`
    and `Refused::{Unknown, Range}` stay (`limits.rs` is split: `check` moves, `apply` stays).
  - What moves is 13 tests (tests/reconcile.rs) plus `Run` and the fixtures it needs, rebuilt on the given struct.
  - Step tests 7 asks for that no in-crate test has: a revoked/401 token made again (list, delete, create,
    write); a team made, edited, the bot added; a hook made when none and one patched; an event out of order
    giving `Unexpected` in each step (only the bot's draw of the wrong length is tested); an ended machine
    ignoring events; an absent or non-loopback listener ending `Done` after step 5, no secret read;
    paging of tokens, hooks and members (the world pages teams only); a hook-secret draw of another length;
    a page cap of zero giving `Unexpected`; a 401 on the admin naming the given admin name; limits at the bound
    (`backoff == backoff_max`, `backoff_max == deadline - 1` pass; one past refuses) and a zero limit.
- **04.2.** B1, B2, B6, B7 as decided; the org bound in the machine (4.1/4.4 `orgs`; today only the reader
  `read.rs an_entry_past_the_orgs_limit_is_refused`, which stays temper's).
- **S06.3.** The world's 22 tests move over an item store. Add: hooks, tokens and members longer than a page
  (cap 2, 3+ rows; the fuzzy seeds have at most 3 hooks per org); two services on one forge (4.4: own word, bot,
  team, hooks; the other's bot `Unmarked`); a listener off loopback (no hook request, no secret drawn). The world
  needs `Pipe`/`Cut` and `fake_limits` (temper-engine-protocol-forge-world) and `decode_answer`
  (temper-codecs-world): they land in 06.1 or skein-forge's test support first.
- **T04.1.** The mapping test (setup.md 13.1): `temper`, `temper-api`, `temper-git`, `temper-admin`, owner and made
  from `mode`, limits from `setup.*`; the shell's `Item <-> File` in `bound()`/`file_path()`; `Refused::line()` on the
  kit's `Why`; imports in the shell and the world (`temper_forge_setup::reconcile::*`); edges of 11 (B9);
  `temper-forge-setup` keeps `skein-forge` for the pin (prepare.rs, run.rs).
- **S07.2.** No work: all 36 facts of section 8 are in temper's `facts.txt`. Section 8 omits two the machine relies
  on ("a 404 is as good"): `setup-token-delete-of-a-missing-id-is-404`, `setup-hook-delete-of-a-missing-id-is-404`.

## 2. The survey's "not verified"

- **Re-listing the same hook page after a deletion: true.** `hooks.rs`, `Phase::Delete` calls `self.listing`
  with `self.page` unchanged (it advances only in `rows`). Harmless: a delete shifts one row, the first of the
  page, down a page, and that row was already seen; the re-list of the same page picks up the next page's first.
  Untested above page 1: `a_lower_hook_met_later_deletes_the_higher_one_kept` re-lists page 1; the fuzzy seeds
  hold at most two same-URL hooks and one other against a cap of 2. Add to 04.1 (a hook met on page 2).
- **Duplicate-token name race: true, and a refusal.** `tokens.rs`, `rows()` deletes the first row of the name and
  the next phase creates, with no re-list. A second same-named token cannot exist on Forgejo (fact
  `setup-token-of-an-existing-name-is-refused`); a concurrent maker makes the create answer a status:
  `Refused(status)`, step `Token`, subject the token name, nothing written; the next run converges. The forge's
  lock (6.2) is the guard. No test drives it (add: create refused).

## 3. Names the design gives

```
forge-setup.md 4.2: step(machine, event, out); MAX_OUT = 1; Need::{Send{request,token}, Read(item), Write{item,bytes},
  Draw(n), Done, Stop(failure)}; Event::{Begin, Read, Reply, Written, Drawn}; Reply::{Document, Rows, Empty,
  Status(code), Down}
4.4: Failure{forge, step, subject, why}; step: Admin, Bot, Token, Org, Hook; why: Missing, Refused(status), NotAdmin,
  BotIsAdmin, Unmarked, OrgMissing, Down, Unexpected; limits: orgs, deadline, backoff, backoff_max
4.1: given values (struct unnamed): forge number, bot login+email, tokens' names (api, git), orgs, orgs' owner, made
  flag, admin's name, listener address, word, limits; items: admin token, API token, git token, bot id, webhook secret
4.3: marker `<word>:bot`; hook URL `http://<address>/hooks/<number>`; scopes API read:user write:issue
  write:repository, git write:repository read:user; team = bot's login, permission write,
  includes_all_repositories, units code/issues/pulls/actions at write; org owned by owner, private
crates: skein-forge-setup (skein-lib, skein-json, skein-forge); tests/forge/setup (world); tests/forge/conformance
facts: 36 `setup-*`, `delivery-loopback-*`, `api-default-page-cap-50-and-30` (forge-setup.md 8)
temper (setup.md 5, 11): temper-forge-setup, temper-forge-setup-shell, temper-box, temper; word `temper`,
  tokens `temper-api`/`temper-git`, admin `temper-admin`; limits setup.orgs, setup.deadline_ms, setup.backoff_ms,
  setup.backoff_max_ms; files api_token, git_token, user, webhook_secret, admin_token
```

Not named by the design (the session files must choose): the given struct, `Place`/check, the item enum's variants,
module paths. Names in today's code to move: `reconcile::{Need, Event, Reply, Failure, Stage, Why, Reconcile, step,
MAX_OUT, hook_url}`, `limits::{Limits, Refused, Unfit, small, apply, check}`; world crate
`temper-forge-setup-reconcile-world` (`World::{owned, listening, reconcile, put, file, requests, store, store_mut,
secrets_absent}`, `Ended`, `fixtures::{random, assert_bot, assert_org, assert_other_hooks, file_writes,
write_routes}`); tests: tests/reconcile.rs (13), tests/reconcile.rs of the world (21), tests/fuzzy_reconcile.rs (1).

## 4. The outline against the design

- 04.1 "~29 tests move": 13 move. `members` (7), `read` (5), `steps` (4) test temper's configuration and
  steps (`check::checked`, `read`, `steps`), which stay; `limits` (3) are split.
- 04.1's given values "(word, admin login, bot, tokens' names and scopes, mode)" differ from 4.1: scopes are fixed
  by 4.3, not given; "admin login" is the name a failure gives the admin; "mode" is the orgs' owner and the made
  flag; missing are the forge number, orgs, listener address and limits.
- T04 hands "temper:bot": the owner hands the word `temper`; the marker is derived. T04 omits the edge
  `temper-forge-setup -> skein-forge-setup` (B9) and the shell's edges (setup.md 11).
- S06's Needs list S02 and S03, which 06.3 does not use; it uses 06.1's `Pipe`/`Cut` and the codecs-world decoder
  (see S06.3 above), which the outline does not order.
- No order the design forbids; S04 (Needs S01.1, T01.1) is right.
