# Supplement temper-07.s1: temper's own cards, and `jig-client-view` dropped

Added by ruling 394 (the designer's answer of 2026-10-18T19:47Z,
point 4). Read first: temper's `AGENTS.md`; `docs/design/client.md`, 1
("temper calls no renderer of jig's reference UI, and copies none of its
modules, functions or phrase tables"), 3.5, 3.6, 5.1 and 9;
`temper/07-task-pages.md`, "temper's own cards" and "Rules that bind it";
the mockups `docs/design/web/mockups/{inbox,index,task}.html`.

Scope:
- **The cards:** the inbox's, a chat's, the task page's and the confirmations' cards become temper's own, after the mockups.
- **The removals:** 07's Removes, which rulings 391 and 392 kept.

Needs: temper 07.4 (landed `5fd9d29c4`) and jig-15.s1. `temper-repin`
waits for it.

Files:
- every user of `jig-client-view` on temper's `origin/main`. Today:
  - `git grep -l -w jig_client_view -- '*.rs'` prints 32 files, in `crates/temper-client-{domain,view,shell-terminal}` and `tests/client/src/{app,fixtures,translate}.rs`;
  - `git grep -l jig-client-view -- '*Cargo.toml'` prints the workspace's, the three client crates', `crates/temper`'s, `tests/client`'s and `tests/one-process`'s;
- `Cargo.lock`;
- the goldens that show a card.

Shape:
1. **The first commit** moves temper's jig pin alone, to jig-15.s1's landing. Past 60 lines of repairs, counted with `git diff --numstat`, block with the count.
2. **The cards** are temper's nodes, drawn from the documents and `jig_client::facts`:
   - a decision's facts are said in `jig_client::words`, with temper's overrides;
   - executors and outcomes are said in temper's own phrases (`later.md`).
3. **The world:** temper's world gives jig's referee temper's phrasebook, through jig-15.s1's `Settings` field.
4. **The removals:**
   - 07's Removes: the reference UI's child, `Owner::Jig`, `Address::Jig`, `parts.rs`, the forge side's held parts, `Limits.jig`;
   - the `jig_client_view::Limits` in `tests/client/src/translate.rs`.

   With them comes 07.4's world test
   `a_client_closed_on_a_tasks_page_opens_at_the_inbox_again`, which
   `5fd9d29c4` left for the child's removal.

Test shape: each golden that shows a card is recorded again by hand,
listed by file in the report. The stories keep their names, acts and
expected values, except literals the new words rename. Each test under
20 ms (unit) or 0.5 s (world).

Proof: 07.4's Proof (the one-process world's adoption story, and the
process tier's `temper_open_signs_in_prints_the_inbox_and_exits_on_q`),
once.

Done when: 07's two greps print nothing (`temper/07-task-pages.md`,
"Rules that bind it"; README, Done when, 8).

Size: measure it first, by card, before more code. Aim at most 1,200
production lines added (removals not counted); past 2,000 (ruling 389's
stop), block with the count.

Don't: call or copy anything of `jig-client-view`, or land the branch
`client-api/temper-07-4-renderers-draft`.
